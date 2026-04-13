use serde::Serialize;
use std::process::Command;

#[derive(Clone, Serialize)]
pub struct GstLibavInfo {
    pub installed: bool,
    pub distro: String,
    pub distro_name: String,
    pub install_command: String,
    pub plugin_version: Option<String>,
}

const DISTRO_MAP: &[(&str, &str, &str)] = &[
    ("ubuntu", "Ubuntu", "sudo apt install gstreamer1.0-libav"),
    ("debian", "Debian", "sudo apt install gstreamer1.0-libav"),
    ("fedora", "Fedora", "sudo dnf install gstreamer1-libav"),
    ("arch", "Arch Linux", "sudo pacman -S gst-libav"),
    (
        "opensuse-leap",
        "openSUSE Leap",
        "sudo zypper install gstreamer-libav",
    ),
    (
        "opensuse-tumbleweed",
        "openSUSE Tumbleweed",
        "sudo zypper install gstreamer-libav",
    ),
    ("alpine", "Alpine Linux", "sudo apk add gst-libav"),
];

fn get_linux_distro() -> Option<String> {
    let content = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if line.starts_with("ID=") {
            return Some(line[3..].trim_matches('"').to_string());
        }
    }
    None
}

fn get_distro_info(distro_id: &str) -> (&str, &str) {
    for (id, name, cmd) in DISTRO_MAP {
        if distro_id.starts_with(id) {
            return (name, cmd);
        }
    }
    ("Linux", "Please check your distro's package manager")
}

fn check_gst_libav() -> Option<String> {
    let output = Command::new("gst-inspect-1.0")
        .arg("libav")
        .output()
        .map_err(|e| eprintln!("[GstCheck] Failed to execute gst-inspect-1.0: {}", e))
        .ok()?;

    if !output.status.success() {
        eprintln!("[GstCheck] gst-inspect-1.0 libav returned non-zero status");
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("Version") {
            let version = line.split_whitespace().nth(1)?;
            eprintln!("[GstCheck] Found libav version: {}", version);
            return Some(version.to_string());
        }
    }
    eprintln!("[GstCheck] gst-inspect-1.0 libav succeeded but no version found");
    Some("installed".to_string())
}

pub fn check_gst_libav_status() -> GstLibavInfo {
    let distro_id = get_linux_distro().unwrap_or_else(|| "unknown".to_string());
    let (distro_name, default_cmd) = get_distro_info(&distro_id);
    let distro_id_cloned = distro_id.clone();

    if let Some(version) = check_gst_libav() {
        GstLibavInfo {
            installed: true,
            distro: distro_id_cloned,
            distro_name: distro_name.to_string(),
            install_command: default_cmd.to_string(),
            plugin_version: Some(version),
        }
    } else {
        GstLibavInfo {
            installed: false,
            distro: distro_id_cloned,
            distro_name: distro_name.to_string(),
            install_command: default_cmd.to_string(),
            plugin_version: None,
        }
    }
}
