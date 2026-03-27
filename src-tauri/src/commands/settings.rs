use tauri::State;
use crate::db::DbState;
use crate::db::migrations::reset_all_data;
use crate::error::AppResult;
use crate::helpers::timestamps::now_utc;

#[tauri::command]
pub fn get_setting(key: String, state: State<'_, DbState>) -> AppResult<Option<String>> {
    let conn = state.read_conn.lock().unwrap();
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(val) => Ok(Some(val)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub fn set_setting(key: String, value: String, state: State<'_, DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        rusqlite::params![key, value, now],
    )?;
    Ok(())
}

/// Delete ALL user data and re-seed defaults. Irreversible.
#[tauri::command]
pub fn delete_all_data(state: State<'_, DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    reset_all_data(&conn)?;
    Ok(())
}
