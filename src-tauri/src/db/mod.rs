pub mod migrations;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use crate::error::AppResult;
use migrations::run_migrations;

/// Holds the database connections.
/// write_conn is mutex-protected — only one writer at a time.
/// read_conn is for queries and dashboard aggregation (WAL allows concurrent reads).
pub struct DbState {
    pub write_conn: Mutex<Connection>,
    pub read_conn: Mutex<Connection>,
}

/// Opens the database, runs migrations, and returns a DbState.
/// DB file lives in the Tauri app data directory.
pub fn init_db(app: &AppHandle) -> AppResult<DbState> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");

    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("privacyfinance.db");

    let write_conn = Connection::open(&db_path)?;
    write_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&write_conn)?;

    let read_conn = Connection::open(&db_path)?;
    read_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    Ok(DbState {
        write_conn: Mutex::new(write_conn),
        read_conn: Mutex::new(read_conn),
    })
}
