/*!
 * Portions of this file are based on code from `wyhaya/tauri-plugin-theme`.
 *
 * Credits to Alexandru Bereghici: https://github.com/wyhaya/tauri-plugin-theme
 */

use crate::{save_theme_value, Theme};
use tauri::{AppHandle, Runtime};

#[tauri::command(rename_all = "snake_case")]
pub fn set_theme<R: Runtime>(app: AppHandle<R>, theme: Theme) -> Result<(), &'static str> {
    save_theme_value(theme, app.clone());
    app.restart();
    Ok(())
}
