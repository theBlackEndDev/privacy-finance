use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::agent::ScopeGrant;
use crate::models::scope::Scope;

#[tauri::command]
pub fn get_agent_scopes(agent_id: String, state: State<DbState>) -> AppResult<Vec<ScopeGrant>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, scope, access_mode FROM agent_scope_grants WHERE agent_id = ?1"
    )?;
    let grants = stmt
        .query_map(rusqlite::params![agent_id], |row| {
            Ok(ScopeGrant {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                scope: row.get(2)?,
                access_mode: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(grants)
}

#[tauri::command]
pub fn set_agent_scope(
    agent_id: String,
    scope: String,
    access_mode: String,
    state: State<DbState>,
) -> AppResult<ScopeGrant> {
    // Validate the scope string parses correctly
    let _parsed = Scope::parse(&scope)?;

    let valid_modes = ["always", "ask_each_time", "never"];
    if !valid_modes.contains(&access_mode.as_str()) {
        return Err(crate::error::AppError::Validation(format!(
            "invalid access_mode: {access_mode}. Must be one of: {}",
            valid_modes.join(", ")
        )));
    }

    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();

    // UPSERT: insert or replace on (agent_id, scope) unique constraint
    conn.execute(
        "INSERT INTO agent_scope_grants (id, agent_id, scope, access_mode)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(agent_id, scope) DO UPDATE SET access_mode = ?4",
        rusqlite::params![id, agent_id, scope, access_mode],
    )?;

    // Return the current state (may have used existing id on conflict)
    let grant = conn.query_row(
        "SELECT id, agent_id, scope, access_mode FROM agent_scope_grants WHERE agent_id = ?1 AND scope = ?2",
        rusqlite::params![agent_id, scope],
        |row| {
            Ok(ScopeGrant {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                scope: row.get(2)?,
                access_mode: row.get(3)?,
            })
        },
    )?;

    Ok(grant)
}
