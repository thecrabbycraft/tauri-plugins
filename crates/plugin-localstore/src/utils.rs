// Copyright © 2024 Crabby Craft - All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fs;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

/// Gets the path to the application's database file.
///
/// This function generates the path to the application's database file based on
/// the application's package name and whether the application is in debug mode.
/// If a file name is provided, it will be used instead of the package name.
///
/// # Arguments
///
/// * `handle` - The application handle.
/// * `file_name` - An optional file name for the database file.
///
/// # Returns
///
/// The path to the application's database file.
pub fn get_db_path<R: Runtime>(handle: &AppHandle<R>, file_name: Option<String>) -> tauri::Result<PathBuf> {
    let pkg_name = handle.package_info().crate_name.to_string();
    let debug = cfg!(debug_assertions);

    let db_file_name = match file_name {
        Some(file) if debug => format!("{file}-debug.db"),
        Some(file) => format!("{file}.db"),
        None if debug => format!("{pkg_name}-debug.db"),
        None => format!("{pkg_name}.db"),
    };

    let db_file_path = handle
        .path()
        .resolve(db_file_name, BaseDirectory::AppConfig)
        .expect("failed to get db file path");

    if let Some(parent) = db_file_path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|err| panic!("Failed to create application data directory: {}", err));
    }

    Ok(db_file_path)
}
