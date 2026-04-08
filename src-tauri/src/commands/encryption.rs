use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::encryption::EncryptionStatus;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
use zeroize::Zeroize;

/// Resolves the DB path from the Tauri app data directory.
fn db_path(app: &AppHandle) -> AppResult<PathBuf> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|_| AppError::Validation("failed to resolve app data dir".into()))?;
    Ok(app_dir.join("privacyfinance.db"))
}

/// Tests whether a SQLite file is encrypted by checking its 16-byte header.
fn is_db_encrypted(path: &Path) -> AppResult<bool> {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(path) else { return Ok(false) };
    let mut buf = [0u8; 16];
    match f.read_exact(&mut buf) {
        Ok(()) => Ok(&buf != b"SQLite format 3\0"),
        Err(_) => Ok(false),
    }
}

/// Escapes a string for use in SQLCipher PRAGMA values.
fn escape_pragma(s: &str) -> String {
    s.replace('\'', "''")
}

/// Escapes a path for use in SQLite ATTACH DATABASE statements.
fn escape_path(p: &Path) -> String {
    p.to_string_lossy().replace('\'', "''")
}

/// Opens a connection, applies the key (if any), and sets standard PRAGMAs.
fn open_keyed_conn(path: &Path, key: Option<&str>) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    if let Some(k) = key {
        conn.execute_batch(&format!("PRAGMA key = '{}';", k))?;
    }
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

/// Replaces both managed DB connections with fresh ones.
fn replace_connections(state: &State<DbState>, path: &Path, key: Option<&str>) -> AppResult<()> {
    let wc = open_keyed_conn(path, key)?;
    let rc = open_keyed_conn(path, key)?;
    {
        let mut w = state.write_conn.lock().unwrap();
        *w = wc;
    }
    {
        let mut r = state.read_conn.lock().unwrap();
        *r = rc;
    }
    Ok(())
}

#[tauri::command]
pub fn check_encryption_status(app: AppHandle) -> AppResult<EncryptionStatus> {
    let path = db_path(&app)?;
    let encrypted = is_db_encrypted(&path)?;
    Ok(EncryptionStatus {
        is_encrypted: encrypted,
    })
}

#[tauri::command]
pub fn enable_encryption(app: AppHandle, mut password: String) -> AppResult<()> {
    if password.is_empty() {
        return Err(AppError::Validation("Password cannot be empty".into()));
    }

    let path = db_path(&app)?;

    if is_db_encrypted(&path)? {
        password.zeroize();
        return Err(AppError::Validation("Database is already encrypted".into()));
    }

    let tmp_path = path.with_extension("db.enc");
    let mut escaped_pw = escape_pragma(&password);
    password.zeroize();

    // Open the plaintext DB and export to an encrypted copy
    let plain_conn = Connection::open(&path)?;
    plain_conn.execute_batch(&format!(
        "ATTACH DATABASE '{}' AS encrypted KEY '{}'",
        escape_path(&tmp_path),
        escaped_pw
    ))?;
    plain_conn.execute_batch(
        "SELECT sqlcipher_export('encrypted'); DETACH DATABASE encrypted;"
    )?;

    // Atomic swap: replace plaintext with encrypted
    std::fs::rename(&tmp_path, &path)?;

    let state: State<DbState> = app.state();
    let result = replace_connections(&state, &path, Some(&escaped_pw));
    escaped_pw.zeroize();

    result
}

#[tauri::command]
pub fn unlock_database(app: AppHandle, mut password: String) -> AppResult<()> {
    let path = db_path(&app)?;

    if !is_db_encrypted(&path)? {
        password.zeroize();
        return Err(AppError::Validation("Database is not encrypted".into()));
    }

    let mut escaped_pw = escape_pragma(&password);
    password.zeroize();

    // Verify the password works before replacing connections
    let test_conn = Connection::open(&path)?;
    test_conn.execute_batch(&format!("PRAGMA key = '{}';", escaped_pw))?;
    test_conn
        .execute_batch("SELECT count(*) FROM sqlite_master;")
        .map_err(|_| {
            escaped_pw.zeroize();
            AppError::Validation("Incorrect password".into())
        })?;
    drop(test_conn);

    let state: State<DbState> = app.state();
    let result = replace_connections(&state, &path, Some(&escaped_pw));
    escaped_pw.zeroize();

    result
}

#[tauri::command]
pub fn change_password(app: AppHandle, mut current_password: String, mut new_password: String) -> AppResult<()> {
    if new_password.is_empty() {
        current_password.zeroize();
        new_password.zeroize();
        return Err(AppError::Validation("New password cannot be empty".into()));
    }

    let path = db_path(&app)?;

    if !is_db_encrypted(&path)? {
        current_password.zeroize();
        new_password.zeroize();
        return Err(AppError::Validation("Database is not encrypted".into()));
    }

    let mut escaped_current = escape_pragma(&current_password);
    let mut escaped_new = escape_pragma(&new_password);
    current_password.zeroize();
    new_password.zeroize();

    // Open with current password, verify, and rekey
    let conn = Connection::open(&path)?;
    conn.execute_batch(&format!("PRAGMA key = '{}';", escaped_current))?;
    escaped_current.zeroize();
    conn.execute_batch("SELECT count(*) FROM sqlite_master;")
        .map_err(|_| {
            escaped_new.zeroize();
            AppError::Validation("Current password is incorrect".into())
        })?;
    conn.execute_batch(&format!("PRAGMA rekey = '{}';", escaped_new))?;
    drop(conn);

    let state: State<DbState> = app.state();
    let result = replace_connections(&state, &path, Some(&escaped_new));
    escaped_new.zeroize();

    result
}

#[tauri::command]
pub fn disable_encryption(app: AppHandle, mut password: String) -> AppResult<()> {
    let path = db_path(&app)?;

    if !is_db_encrypted(&path)? {
        password.zeroize();
        return Err(AppError::Validation("Database is not encrypted".into()));
    }

    let mut escaped_pw = escape_pragma(&password);
    password.zeroize();
    let tmp_path = path.with_extension("db.plain");

    // Open encrypted DB and export to plaintext
    let enc_conn = Connection::open(&path)?;
    enc_conn.execute_batch(&format!("PRAGMA key = '{}';", escaped_pw))?;
    escaped_pw.zeroize();
    enc_conn.execute_batch("SELECT count(*) FROM sqlite_master;")
        .map_err(|_| AppError::Validation("Incorrect password".into()))?;

    enc_conn.execute_batch(&format!(
        "ATTACH DATABASE '{}' AS plaintext KEY ''",
        escape_path(&tmp_path)
    ))?;
    enc_conn.execute_batch(
        "SELECT sqlcipher_export('plaintext'); DETACH DATABASE plaintext;"
    )?;

    // Atomic swap
    std::fs::rename(&tmp_path, &path)?;

    let state: State<DbState> = app.state();
    replace_connections(&state, &path, None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("privacyfinance.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE test_data (id INTEGER PRIMARY KEY, value TEXT);
             INSERT INTO test_data VALUES (1, 'hello');
             INSERT INTO test_data VALUES (2, 'world');"
        ).unwrap();
        path
    }

    fn encrypt_test_db(path: &Path, password: &str) {
        let conn = Connection::open(path).unwrap();
        let enc_path = path.with_extension("db.enc");
        conn.execute_batch(&format!(
            "ATTACH DATABASE '{}' AS encrypted KEY '{}'",
            enc_path.to_string_lossy(),
            password
        )).unwrap();
        conn.execute_batch("SELECT sqlcipher_export('encrypted'); DETACH DATABASE encrypted;").unwrap();
        drop(conn);
        std::fs::rename(&enc_path, path).unwrap();
    }

    #[test]
    fn test_is_db_encrypted_plaintext() {
        let dir = TempDir::new().unwrap();
        let path = create_test_db(&dir);
        assert!(!is_db_encrypted(&path).unwrap());
    }

    #[test]
    fn test_enable_makes_db_unreadable() {
        let dir = TempDir::new().unwrap();
        let path = create_test_db(&dir);
        encrypt_test_db(&path, "testpass123");

        assert!(is_db_encrypted(&path).unwrap());

        // Plaintext read should fail
        let plain_conn = Connection::open(&path).unwrap();
        assert!(plain_conn.execute_batch("SELECT count(*) FROM sqlite_master;").is_err());
    }

    #[test]
    fn test_unlock_succeeds_with_correct_password() {
        let dir = TempDir::new().unwrap();
        let path = create_test_db(&dir);
        encrypt_test_db(&path, "mypassword");

        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA key = 'mypassword';").unwrap();
        assert!(conn.execute_batch("SELECT count(*) FROM sqlite_master;").is_ok());
    }

    #[test]
    fn test_unlock_fails_with_wrong_password() {
        let dir = TempDir::new().unwrap();
        let path = create_test_db(&dir);
        encrypt_test_db(&path, "correctpass");

        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA key = 'wrongpass';").unwrap();
        assert!(conn.execute_batch("SELECT count(*) FROM sqlite_master;").is_err());
    }

    #[test]
    fn test_change_password_allows_new_only() {
        let dir = TempDir::new().unwrap();
        let path = create_test_db(&dir);
        encrypt_test_db(&path, "original");

        // Change password
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA key = 'original';").unwrap();
        conn.execute_batch("SELECT count(*) FROM sqlite_master;").unwrap();
        conn.execute_batch("PRAGMA rekey = 'newpassword';").unwrap();
        drop(conn);

        // Old password should fail
        let old_conn = Connection::open(&path).unwrap();
        old_conn.execute_batch("PRAGMA key = 'original';").unwrap();
        assert!(old_conn.execute_batch("SELECT count(*) FROM sqlite_master;").is_err());

        // New password should work
        let new_conn = Connection::open(&path).unwrap();
        new_conn.execute_batch("PRAGMA key = 'newpassword';").unwrap();
        assert!(new_conn.execute_batch("SELECT count(*) FROM sqlite_master;").is_ok());
    }

    #[test]
    fn test_disable_encryption_restores_plaintext() {
        let dir = TempDir::new().unwrap();
        let path = create_test_db(&dir);
        encrypt_test_db(&path, "temppass");
        assert!(is_db_encrypted(&path).unwrap());

        // Decrypt
        let enc_conn = Connection::open(&path).unwrap();
        enc_conn.execute_batch("PRAGMA key = 'temppass';").unwrap();
        enc_conn.execute_batch("SELECT count(*) FROM sqlite_master;").unwrap();
        let plain_path = path.with_extension("db.plain");
        enc_conn.execute_batch(&format!(
            "ATTACH DATABASE '{}' AS plaintext KEY ''",
            plain_path.to_string_lossy()
        )).unwrap();
        enc_conn.execute_batch("SELECT sqlcipher_export('plaintext'); DETACH DATABASE plaintext;").unwrap();
        drop(enc_conn);
        std::fs::rename(&plain_path, &path).unwrap();

        assert!(!is_db_encrypted(&path).unwrap());
        let plain_conn = Connection::open(&path).unwrap();
        assert!(plain_conn.execute_batch("SELECT count(*) FROM sqlite_master;").is_ok());
    }
}
