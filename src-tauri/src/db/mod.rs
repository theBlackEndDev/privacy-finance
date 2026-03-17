pub mod migrations;
mod tests;

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

/// Tests whether a SQLite file is encrypted by checking its 16-byte header.
fn is_db_encrypted(path: &std::path::Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(path) else { return false };
    let mut buf = [0u8; 16];
    match f.read_exact(&mut buf) {
        Ok(()) => &buf != b"SQLite format 3\0",
        Err(_) => false,
    }
}

/// Opens the database, runs migrations, and returns a DbState.
/// DB file lives in the Tauri app data directory.
/// If the DB is encrypted, connections are opened but migrations are skipped
/// until the user provides the password via unlock_database.
pub fn init_db(app: &AppHandle) -> AppResult<DbState> {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");

    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("privacyfinance.db");

    if is_db_encrypted(&db_path) {
        // DB is encrypted — open connections but don't run migrations.
        // The unlock_database command will set the key and reinitialize.
        let write_conn = Connection::open(&db_path)?;
        let read_conn = Connection::open(&db_path)?;
        return Ok(DbState {
            write_conn: Mutex::new(write_conn),
            read_conn: Mutex::new(read_conn),
        });
    }

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
