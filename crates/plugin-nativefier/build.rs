// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

const COMMANDS: &[&str] = &[
    "get_theme",
    "list_font_mono",
    "list_font_sans",
    "open_data_directory",
    "open_in_browser",
    "open_log_directory",
    "open_log_file",
    "set_theme",
    "toggle_devtools",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .build();
}
