/*!
 * Portions of this file are based on code from `wyhaya/tauri-plugin-theme`.
 *
 * Credits to Alexandru Bereghici: https://github.com/wyhaya/tauri-plugin-theme
 */

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Auto,
    Light,
    Dark,
}

impl From<&str> for Theme {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::Auto,
        }
    }
}

impl ToString for Theme {
    fn to_string(&self) -> String {
        match self {
            Theme::Auto => "auto".into(),
            Theme::Light => "light".into(),
            Theme::Dark => "dark".into(),
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
// pub fn get_theme(db: tauri::State<AppState>) -> tauri::Result<Theme> {
pub fn get_theme() -> tauri::Result<Theme> {
    // Ok(saved_theme_value(db))
    Ok(Theme::Auto)
}

// pub fn saved_theme_value(_state: tauri::State<AppState>) -> Theme {
pub fn saved_theme_value() -> Theme {
    // setting::AppSettings::default().app_theme
    Theme::Auto
}

pub fn save_theme_value<R: Runtime>(theme: Theme, _app: AppHandle<R>) {
    // setting::save_setting("theme", &theme.to_string(), app)
    format!("theme: {}", theme.to_string());
}
