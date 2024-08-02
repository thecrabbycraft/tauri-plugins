/*!
 * Portions of this file are based on code from `chrisllontop/keyv-rust`.
 * MIT Licensed, Copyright (c) 2023 Christian Llontop.
 *
 * Credits to Alexandru Bereghici: https://github.com/chrisllontop/keyv-rust
 */

use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;

use super::{Store, StoreError, DEFAULT_NAMESPACE_NAME};

/// Builder for creating a `KeyvStore`.
///
/// This builder allows for configuring a `KeyvStore` with custom
/// settings such as a specific database file URI and a table name.
/// It provides a flexible way to initialize the store depending on the
/// application's requirements.
///
/// # Examples
///
/// ## Initializing with a Database File URI
///
/// ```rust,no_run
/// # use keyv::{KeyvStoreBuilder};
/// # #[tokio::main]
/// # async fn main(){
/// let store = KeyvStoreBuilder::new()
///     .uri("sqlite::memory:")
///     .table_name("custom_table_name")
///     .build()
///     .unwrap();
///  }
/// ```
///
/// ## Using an Existing Connection Pool
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use keyv::{KeyvStoreBuilder};
/// # use rusqlite::Connection;
///
/// # #[tokio::main]
/// # async fn main() {
/// let conn: Arc<Connnection> = Arc::new(Connection::open(path).unwrap());
///
/// let store = KeyvStoreBuilder::new()
///     .connnection(conn)
///     .table_name("custom_table_name")
///     .build()
///     .unwrap();
/// }
/// ```
pub struct KeyvStoreBuilder {
    uri: Option<PathBuf>,
    connnection: Option<Arc<Connection>>,
    table_name: Option<String>,
}

impl KeyvStoreBuilder {
    pub fn new() -> Self {
        Self {
            uri: None,
            connnection: None,
            table_name: None,
        }
    }

    /// Sets the table name for the `KeyvStore`.
    ///
    /// This method configures the table name to be used by the store. If not set,
    /// `DEFAULT_TABLE_NAME` from the configuration will be used.
    pub fn table_name<S: Into<String>>(mut self, table: S) -> Self {
        self.table_name = Some(table.into());
        self
    }

    /// Sets the database URI for connecting to the SQLite database.
    ///
    /// This method configures the database URI. It's required if no existing connection is provided.
    pub fn uri<S: Into<PathBuf>>(mut self, uri: S) -> Self {
        self.uri = Some(uri.into());
        self
    }

    /// Uses an existing connection for the `KeyvStore`.
    ///
    /// This method allows for using an already configured `Pool`. If set,
    /// the `uri` option is ignored.
    pub fn connnection(mut self, connnection: Arc<Connection>) -> Self {
        self.connnection = Some(connnection);
        self
    }

    /// Builds the `KeyvStore` based on the provided configurations.
    ///
    /// Finalizes the builder and creates an `KeyvStore` instance.
    /// It requires either a database URI or an existing connection to be set.
    ///
    /// # Returns
    /// This method returns a `Result` which, on success, contains the initialized `KeyvStore`.
    /// On failure, it returns a `StoreError` indicating what went wrong during the initialization.
    pub fn build(self) -> Result<KeyvStore, StoreError> {
        let connnection = match self.connnection {
            Some(connnection) => connnection,
            None => {
                let path = self
                    .uri
                    .expect("KeyvStore requires either a URI or an existing connnection to be set");

                let db = Connection::open(path)
                    .map_err(|_| StoreError::ConnectionError("Failed to create database connnection".to_string()))?;

                Arc::new(db)
            }
        };

        let table_name = self.table_name.unwrap_or_else(|| {
            log::warn!("Table name not set, using default table name");
            DEFAULT_NAMESPACE_NAME.to_string()
        });

        Ok(KeyvStore {
            connnection,
            table_name,
        })
    }
}

pub struct KeyvStore {
    pub(crate) connnection: Arc<Connection>,
    pub(crate) table_name: String,
}

impl KeyvStore {
    fn get_table_name(&self) -> String {
        self.table_name.clone()
    }
}

impl Store for KeyvStore {
    fn initialize(&self) -> Result<(), StoreError> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (key TEXT PRIMARY KEY, value TEXT NOT NULL, UNIQUE(key)) STRICT;
            CREATE INDEX IF NOT EXISTS {table_name}_key_idx ON {table_name} (key);",
            table_name = self.get_table_name()
        );

        let conn = &*self.connnection;

        conn.execute_batch(&query).map_err(|e| {
            StoreError::QueryError(format!("Failed to initialize the database table: {}", e.to_string()))
        })?;

        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<Value>, StoreError> {
        let query = format!("SELECT value FROM {} WHERE key = ?1 LIMIT 1", self.get_table_name());

        let conn = &*self.connnection;

        let result: String = conn
            .query_row_and_then(&query, [key], |row| row.get(0))
            .map_err(|_| StoreError::QueryError("Failed to fetch the value".to_string()))?;

        let value = serde_json::from_str(&result).ok().flatten();

        Ok(value)
    }

    fn set(&self, key: &str, value: Value, _ttl: Option<u64>) -> Result<(), StoreError> {
        let value_str = serde_json::to_string(&value).map_err(|e| StoreError::SerializationError { source: e })?;

        let query = format!(
            "INSERT INTO {} (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = EXCLUDED.value",
            self.get_table_name()
        );

        let conn = &*self.connnection;

        let mut stmt = conn
            .prepare(&query)
            .map_err(|_| StoreError::QueryError("Failed to set the statement".to_string()))?;

        stmt.execute([key, &value_str])
            .map_err(|_| StoreError::QueryError("Failed to set the value".to_string()))?;

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StoreError> {
        let query = format!("DELETE FROM {} WHERE key = ?1", self.get_table_name());

        let conn = &*self.connnection;

        let mut stmt = conn
            .prepare(&query)
            .map_err(|_| StoreError::QueryError("Failed to set the statement".to_string()))?;

        stmt.execute([key])
            .map_err(|_| StoreError::QueryError("Failed to remove the key".to_string()))?;

        Ok(())
    }

    fn remove_many(&self, keys: &[&str]) -> Result<(), StoreError> {
        // let query = format!(
        //     "DELETE FROM {} WHERE key IN ({})",
        //     self.get_table_name(),
        //     keys.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
        // );

        // let mut query = sqlx::query(&query);
        // for key in keys {
        //     query = query.bind(key);
        // }

        // query
        //     .execute(&*self.connnection)
        //
        //     .map_err(|e| StoreError::QueryError(format!("Failed to remove the keys: {}", e.to_string())))?;

        let conn = &*self.connnection;

        log::debug!("Removing keys: {:?} {:?}", keys, conn);

        Ok(())
    }

    fn clear(&self) -> Result<(), StoreError> {
        let query = format!("DELETE FROM {}", self.get_table_name());
        let conn = &*self.connnection;

        conn.execute(&query, [])
            .map_err(|_| StoreError::QueryError("Failed to clear the table".to_string()))?;

        Ok(())
    }
}
