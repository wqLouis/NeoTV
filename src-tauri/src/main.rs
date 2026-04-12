// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("GDK_BACKEND", "x11");
        std::env::set_var("GTK_BACKEND", "x11");
        std::env::remove_var("WAYLAND_DISPLAY");
    }
    app_lib::run();
}
