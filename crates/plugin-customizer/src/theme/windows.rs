/*!
 * Portions of this file are based on code from `wyhaya/tauri-plugin-theme`.
 *
 * Credits to Alexandru Bereghici: https://github.com/wyhaya/tauri-plugin-theme
 */

use super::{save_theme_value, Theme};
use tauri::{AppHandle, Runtime};

#[tauri::command(rename_all = "snake_case")]
pub fn set_theme<R: Runtime>(app: AppHandle<R>, theme: Theme) -> Result<(), &'static str> {
    save_theme_value(theme, app.clone());
    println!("Theme saved, attempting to restart application...");
    if let Err(_) = app.restart() {
        println!("Failed to restart application");
        return Err("Failed to restart application");
    }
    println!("Application restarted successfully");
    Ok(())
}
