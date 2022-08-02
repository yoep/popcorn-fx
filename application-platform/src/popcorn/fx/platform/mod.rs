pub mod platform_info;
pub mod platform;
#[cfg(target_os = "windows")]
pub mod platform_win;
#[cfg(target_os = "macos")]
pub mod platform_mac;