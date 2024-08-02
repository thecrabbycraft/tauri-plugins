// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use tauri::{Runtime, WebviewWindow};

#[tauri::command(rename_all = "snake_case")]
pub fn toggle_devtools<R: Runtime>(window: WebviewWindow<R>) {
    if !window.is_devtools_open() {
        window.open_devtools()
    } else if window.is_devtools_open() {
        window.close_devtools()
    }
}
