// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde_json::value::Value as JsonValue;
use tauri::{AppHandle, Runtime, WebviewWindow};

use crate::utils;

#[tauri::command(rename_all = "snake_case")]
pub fn toggle_devtools<R: Runtime>(window: WebviewWindow<R>) {
    if !window.is_devtools_open() {
        window.open_devtools()
    } else if window.is_devtools_open() {
        window.close_devtools()
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn open_log_file<R: Runtime>(handle: AppHandle<R>, file_name: &str) {
    utils::open_log_file(&handle, file_name);
}

#[tauri::command(rename_all = "snake_case")]
pub fn open_log_directory<R: Runtime>(handle: AppHandle<R>) {
    utils::open_log_directory(&handle);
}

#[tauri::command(rename_all = "snake_case")]
pub fn open_data_directory<R: Runtime>(handle: AppHandle<R>) {
    utils::open_data_directory(&handle)
}

#[tauri::command(rename_all = "snake_case")]
pub fn open_in_browser(url: &str) {
    utils::open_in_browser(url);
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_font_sans() -> tauri::Result<Vec<JsonValue>> {
    utils::list_fonts(false)
}

#[tauri::command(rename_all = "snake_case")]
pub fn list_font_mono() -> tauri::Result<Vec<JsonValue>> {
    utils::list_fonts(true)
}
