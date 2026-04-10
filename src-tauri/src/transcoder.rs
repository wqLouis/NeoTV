use once_cell::sync::Lazy;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::m3u8;

#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub url: String,
    pub port: u16,
    pub duration: Option<f64>,
}

pub struct TranscoderManager {
    streams: std::collections::HashMap<String, StreamHandle>,
    base_port: u16,
}

struct StreamHandle {
    running: Arc<AtomicBool>,
}

impl TranscoderManager {
    pub fn new() -> Self {
        Self {
            streams: std::collections::HashMap::new(),
            base_port: 9876,
        }
    }

    pub fn find_available_port(&mut self) -> Result<u16, String> {
        for port in self.base_port..(self.base_port + 100) {
            if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                self.base_port = port + 1;
                return Ok(port);
            }
        }
        Err("No available port found".to_string())
    }

    pub fn check_system(&self) -> (bool, bool, Option<String>) {
        let ffmpeg_path = find_ffmpeg().ok();
        let vaapi = ffmpeg_path.as_ref()
            .map(|p| check_vaapi_support(p))
            .unwrap_or(false);
        let nvenc = check_nvenc_support();
        eprintln!("[Transcoder] System check: vaapi={}, nvenc={}", vaapi, nvenc);
        (vaapi || nvenc, ffmpeg_path.is_some(), ffmpeg_path)
    }

    pub fn stop_stream(&mut self, id: &str) {
        if let Some(handle) = self.streams.remove(id) {
            handle.running.store(false, Ordering::SeqCst);
            eprintln!("[Transcoder] Stopped stream: {}", id);
        }
    }

    pub fn stop_all(&mut self) {
        for (id, handle) in self.streams.drain() {
            handle.running.store(false, Ordering::SeqCst);
            eprintln!("[Transcoder] Stopped stream: {}", id);
        }
    }
}

impl Drop for TranscoderManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

pub static TRANSCODER: Lazy<Mutex<TranscoderManager>> =
    Lazy::new(|| Mutex::new(TranscoderManager::new()));

pub fn run_streaming_transcoder(
    id: String,
    m3u8_url: String,
    referer: Option<String>,
) -> Result<StreamInfo, String> {
    let port = {
        let mut manager = TRANSCODER.lock().unwrap();
        manager.find_available_port()?
    };

    let running = Arc::new(AtomicBool::new(true));
    let server_ready = Arc::new(AtomicBool::new(false));

    {
        let mut manager = TRANSCODER.lock().unwrap();
        manager.streams.insert(id.clone(), StreamHandle {
            running: running.clone(),
        });
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let media_info = rt.block_on(async {
        m3u8::fetch_and_process_m3u8(&m3u8_url, true).await
    }).map_err(|e| format!("Failed to fetch m3u8: {}", e.error))?;

    let duration = media_info.duration;
    let processed_m3u8 = media_info.processed_content
        .ok_or_else(|| "No processed content".to_string())?;

    let id_owned = id.clone();
    let running_clone = running.clone();
    let server_ready_clone = server_ready.clone();
    let m3u8_url_owned = m3u8_url.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            if let Err(e) = run_pipeline(running_clone, port, &id_owned, m3u8_url_owned, referer, server_ready_clone, processed_m3u8).await {
                eprintln!("[Transcoder] Pipeline error: {}", e);
            }
        });
    });

    let start = std::time::Instant::now();
    while !server_ready.load(Ordering::SeqCst) {
        if start.elapsed().as_secs() > 10 {
            return Err("Timeout waiting for server to start".to_string());
        }
        thread::sleep(Duration::from_millis(50));
    }

    Ok(StreamInfo {
        url: format!("http://127.0.0.1:{}/{}", port, id),
        port,
        duration,
    })
}

async fn run_pipeline(
    running: Arc<AtomicBool>,
    port: u16,
    id: &str,
    m3u8_url: String,
    referer: Option<String>,
    server_ready: Arc<AtomicBool>,
    processed_content: String,
) -> Result<(), String> {
    eprintln!("[Transcoder] Processing m3u8: {}", m3u8_url);

    let segments = parse_segments_from_m3u8(&processed_content)?;

    if segments.is_empty() {
        return Err("No segments found in processed m3u8".to_string());
    }

    eprintln!("[Transcoder] Found {} segments, starting pipeline", segments.len());
    eprintln!("[Transcoder] First segment URL: {}", segments[0]);

    let ffmpeg_path = find_ffmpeg()?;
    let has_vaapi = check_vaapi_support(&ffmpeg_path);
    let has_nvenc = check_nvenc_support();

    eprintln!("[Transcoder] ffmpeg: {}, vaapi: {}, nvenc: {}", ffmpeg_path, has_vaapi, has_nvenc);

    let use_vaapi = has_vaapi && test_vaapi_works();
    let use_nvenc = !use_vaapi && has_nvenc && test_nvenc_works();

    let mut cmd = Command::new(&ffmpeg_path);
    cmd.arg("-y")
       .arg("-hide_banner")
       .arg("-loglevel")
       .arg("warning")
       .arg("-f")
       .arg("mpegts")
       .arg("-i")
       .arg("pipe:0")
       .arg("-fflags")
       .arg("+genpts+nobuffer")
       .arg("-max_delay")
       .arg("0")
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    if use_vaapi {
        eprintln!("[Transcoder] Using VA-API");
        cmd.arg("-vaapi_device")
           .arg("/dev/dri/renderD128")
           .arg("-vf")
           .arg("format=nv12,hwupload")
           .arg("-c:v")
           .arg("h264_vaapi")
           .arg("-b:v")
           .arg("0")
           .arg("-maxrate")
           .arg("8M")
           .arg("-profile:v")
           .arg("high");
    } else if use_nvenc {
        eprintln!("[Transcoder] Using NVENC (NVIDIA)");
        cmd.arg("-c:v")
           .arg("h264_nvenc")
           .arg("-preset")
           .arg("p3")
           .arg("-b:v")
           .arg("0")
           .arg("-maxrate")
           .arg("8M")
           .arg("-profile:v")
           .arg("high");
    } else {
        eprintln!("[Transcoder] Using libx264 (software)");
        cmd.arg("-c:v")
           .arg("libx264")
           .arg("-preset")
           .arg("ultrafast")
           .arg("-crf")
           .arg("23")
           .arg("-profile:v")
           .arg("high");
    }

    cmd.arg("-c:a")
       .arg("aac")
       .arg("-b:a")
       .arg("128k")
       .arg("-movflags")
       .arg("+cmaf+frag_keyframe")
       .arg("-f")
       .arg("mp4")
       .arg("pipe:1");

    eprintln!("[Transcoder] Spawning ffmpeg...");
    eprintln!("[Transcoder] Full ffmpeg args: {:?}", cmd);

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Transcoder] Failed to spawn ffmpeg: {}", e);
            return Err(format!("Failed to start ffmpeg: {}", e));
        }
    };

    eprintln!("[Transcoder] ffmpeg spawned with pid {:?}", child.id());

    let child_stderr = child.stderr.take();
    let child_id = id.to_string();

    std::thread::spawn(move || {
        if let Some(mut stderr) = child_stderr {
            let mut err_buf = String::new();
            use std::io::Read;
            if stderr.read_to_string(&mut err_buf).is_ok() && !err_buf.is_empty() {
                eprintln!("[Transcoder][{}] ffmpeg stderr: {}", child_id, err_buf);
            }
        }
    });

    let mut child_stdin = match child.stdin.take() {
        Some(s) => s,
        None => {
            eprintln!("[Transcoder] stdin is None after spawn!");
            return Err("Failed to capture stdin".to_string());
        }
    };

    let mut child_stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            return Err("Failed to capture stdout".to_string());
        }
    };

    let running_http = running.clone();
    let port_http = port;
    let server_ready_http = server_ready.clone();

    let _http_thread = std::thread::spawn(move || {
        server_ready_http.store(true, Ordering::SeqCst);
        let mut buf = vec![0u8; 65536];
        let listener = match TcpListener::bind(format!("127.0.0.1:{}", port_http)) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[Transcoder] HTTP bind error: {}", e);
                return;
            }
        };
        eprintln!("[Transcoder] HTTP server listening on port {}", port_http);

        for stream in listener.incoming() {
            if !running_http.load(Ordering::SeqCst) {
                break;
            }
            match stream {
                Ok(mut s) => {
                    eprintln!("[Transcoder] Client connected");
                    let response = "HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nConnection: close\r\n\r\n";
                    if let Err(e) = s.write_all(response.as_bytes()) {
                        eprintln!("[Transcoder] Header send error: {}", e);
                        continue;
                    }
                    let mut total_sent = 0;
                    loop {
                        if !running_http.load(Ordering::SeqCst) {
                            break;
                        }
                        match child_stdout.read(&mut buf) {
                            Ok(0) => {
                                eprintln!("[Transcoder] Stream ended, total {} bytes", total_sent);
                                break;
                            }
                            Ok(n) => {
                                total_sent += n;
                                if let Err(e) = s.write_all(&buf[..n]) {
                                    eprintln!("[Transcoder] Write error: {}", e);
                                    break;
                                }
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                thread::sleep(Duration::from_millis(10));
                            }
                            Err(e) => {
                                eprintln!("[Transcoder] Read error: {}", e);
                                break;
                            }
                        }
                    }
                    break;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    eprintln!("[Transcoder] Accept error: {}", e);
                }
            }
        }
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let mut downloaded = 0;
    for (i, url) in segments.iter().enumerate() {
        if !running.load(Ordering::SeqCst) {
            eprintln!("[Transcoder] Stopped");
            break;
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".parse().unwrap());
        headers.insert("Accept", "*/*".parse().unwrap());
        if let Some(ref referer) = referer {
            if let Ok(val) = referer.parse() {
                headers.insert("Referer", val);
            }
        }

        match client.get(url).headers(headers).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.bytes().await {
                        Ok(bytes) => {
                            if i < 3 {
                                eprintln!("[Transcoder] Segment {} size: {}, first bytes: {:02x?}", 
                                    i, bytes.len(), &bytes[..std::cmp::min(16, bytes.len())]);
                            }
                            match child_stdin.write_all(&bytes) {
                                Ok(_) => {
                                    downloaded += 1;
                                }
                                Err(e) => {
                                    eprintln!("[Transcoder] Write error at segment {}: {}", i, e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("[Transcoder] Read error segment {}: {}", i, e);
                        }
                    }
                } else {
                    eprintln!("[Transcoder] Segment {} HTTP error: {}", i, resp.status());
                }
            }
            Err(e) => {
                eprintln!("[Transcoder] Download error segment {}: {}", i, e);
            }
        }

        if downloaded % 10 == 0 || downloaded == segments.len() {
            eprintln!("[Transcoder] Downloaded {}/{}", downloaded, segments.len());
        }
    }

    drop(child_stdin);
    eprintln!("[Transcoder] All segments sent to ffmpeg");

    let _ = child.wait();

    {
        let mut manager = TRANSCODER.lock().unwrap();
        manager.streams.remove(id);
    }

    eprintln!("[Transcoder] Done");
    Ok(())
}

fn parse_segments_from_m3u8(content: &str) -> Result<Vec<String>, String> {
    let mut segments = Vec::new();
    let base_url = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            if trimmed.starts_with("#EXT-X-KEY") || trimmed.starts_with("#EXT-X-MAP") {
                continue;
            }
            continue;
        }

        let url = trimmed.to_string();

        if url.contains(".ts") || url.contains(".m4s") || url.contains(".aac") {
            let full_url = if url.starts_with("http://") || url.starts_with("https://") {
                url
            } else if !base_url.is_empty() {
                format!("{}{}", base_url, url)
            } else {
                url
            };
            segments.push(full_url);
        }
    }

    if segments.is_empty() {
        return Err("No segments found in m3u8".to_string());
    }

    Ok(segments)
}

fn find_ffmpeg() -> Result<String, String> {
    let paths = ["ffmpeg", "/usr/bin/ffmpeg", "/usr/local/bin/ffmpeg"];
    for path in &paths {
        if Command::new(path).arg("-version").output().is_ok() {
            return Ok(path.to_string());
        }
    }
    Err("ffmpeg not found".to_string())
}

fn check_vaapi_support(ffmpeg_path: &str) -> bool {
    let output = Command::new(ffmpeg_path)
        .arg("-hide_banner")
        .arg("-codecs")
        .output()
        .ok();

    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout.contains("h264_vaapi") && std::path::Path::new("/dev/dri/renderD128").exists()
    } else {
        false
    }
}

fn check_nvenc_support() -> bool {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-codecs")
        .output()
        .ok();

    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout.contains("h264_nvenc")
    } else {
        false
    }
}

fn test_vaapi_works() -> bool {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg("testsrc=duration=0.1:size=16x16:rate=1")
        .arg("-vaapi_device")
        .arg("/dev/dri/renderD128")
        .arg("-vf")
        .arg("format=nv12,hwupload")
        .arg("-c:v")
        .arg("h264_vaapi")
        .arg("-f")
        .arg("null")
        .arg("-")
        .output();

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let success = out.status.success() && !stderr.contains("VAAPI");
            eprintln!("[Transcoder] VA-API test: {}", if success { "works" } else { "failed" });
            success
        }
        Err(e) => {
            eprintln!("[Transcoder] VA-API test failed: {}", e);
            false
        }
    }
}

fn test_nvenc_works() -> bool {
    let output = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .arg("-f")
        .arg("lavfi")
        .arg("-i")
        .arg("testsrc=duration=0.1:size=16x16:rate=1")
        .arg("-c:v")
        .arg("h264_nvenc")
        .arg("-f")
        .arg("null")
        .arg("-")
        .output();

    match output {
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let success = out.status.success() && !stderr.contains("Nvenc");
            eprintln!("[Transcoder] NVENC test: {}", if success { "works" } else { "failed" });
            success
        }
        Err(e) => {
            eprintln!("[Transcoder] NVENC test failed: {}", e);
            false
        }
    }
}
