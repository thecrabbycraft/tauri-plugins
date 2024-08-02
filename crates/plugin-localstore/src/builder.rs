// @ref: https://github.com/RandomEngy/tauri-sqlite

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::{fs, path::PathBuf};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager, Runtime};

use super::{Keyv, KeyvStoreBuilder};

/// Initializes the database connection, creating the database file if needed,
/// and running the database migrations if it's out of date.
pub fn initialize<R: Runtime>(app_handle: &AppHandle<R>, migrations: Vec<M>) -> Result<Connection, rusqlite::Error> {
    let db_file_path = get_app_db_path(app_handle, None)
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(tauri::Error::Anyhow(e.into()))))?;

    let mut db = Connection::open(db_file_path)?;

    // Define the migration queries
    // let migrations: Vec<M> = vec![
    //     M::up(include_str!("../../migrations/0001_initialize_tables.sql")),
    //     // M::up(include_str!("../../migrations/0002_foundation_tables.sql")),
    // ];

    let migrate = Migrations::new(migrations);

    // Apply some PRAGMA, often better to do it outside of migrations
    db.pragma_update_and_check(None, "journal_mode", &"WAL", |_| Ok(()))?;

    // Update the database schema, atomically
    migrate.to_latest(&mut db).expect("Failed to migrate database");

    // // Initialize default application settings
    // let count_query = format!("SELECT COUNT(*) FROM {SETTINGS_TABLE_NAME}");
    // let mut stmt = db.prepare(&count_query).expect("Failed to prepare count statement");
    // let count: u32 = stmt
    //     .query_row([], |row| row.get(0))
    //     .expect("Failed to count settings table");

    // if count == 0 {
    //     log::debug!("Populating default application settings");
    //     init_default_setting_values(&handle).expect("Failed to initialize default settings");
    // }

    Ok(db)
}

pub fn get_app_db_path<R: Runtime>(handle: &AppHandle<R>, file_name: Option<&str>) -> tauri::Result<PathBuf> {
    let pkg_name = std::env!("CARGO_PKG_NAME");
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

pub fn get_keyv_instance<R: Runtime>(handle: &AppHandle<R>, table_name: &str) -> tauri::Result<Keyv> {
    let db_file_path = get_app_db_path(&handle, None)?;

    let store = KeyvStoreBuilder::new()
        .uri(db_file_path)
        .table_name(table_name)
        .build()
        .map_err(|e| tauri::Error::Anyhow(e.into()))?;

    Keyv::try_new(store).map_err(|e| tauri::Error::Anyhow(e.into()))
}
