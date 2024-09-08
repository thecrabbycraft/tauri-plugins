/*!
 * Portions of this file are based on code from `wyhaya/tauri-plugin-theme`.
 *
 * Credits to Alexandru Bereghici: https://github.com/wyhaya/tauri-plugin-theme
 */

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub use self::macos::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use self::linux::*;

mod builder;
pub use self::builder::*;
