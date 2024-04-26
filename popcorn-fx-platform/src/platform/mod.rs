pub use platform::*;

mod platform;

#[cfg(target_os = "linux")]
pub mod platform_linux;
#[cfg(target_os = "macos")]
pub mod platform_mac;
#[cfg(target_os = "windows")]
pub mod platform_win;
