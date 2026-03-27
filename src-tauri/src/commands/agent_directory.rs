use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::curated_agents::{get_curated_directory, CuratedAgent};
use crate::helpers::timestamps::now_utc;
use crate::models::agent::Agent;

/// List all curated agents available for one-click install.
#[tauri::command]
pub fn get_curated_agents() -> AppResult<Vec<CuratedAgent>> {
    Ok(get_curated_directory())
}

/// Install a curated agent by its ID — creates it in pending state with
/// pre-populated scopes, ready for the approval modal.
#[tauri::command]
pub fn install_curated_agent(curated_id: String, state: State<DbState>) -> AppResult<Agent> {
    let curated = get_curated_directory()
        .into_iter()
        .find(|a| a.id == curated_id)
        .ok_or_else(|| AppError::NotFound(format!("curated agent {curated_id}")))?;

    let conn = state.write_conn.lock().unwrap();
    let id = curated.id.to_string();
    let now = now_utc();

    // Check if already installed
    let existing: Option<String> = conn
        .query_row("SELECT status FROM agents WHERE id = ?1", rusqlite::params![id], |row| row.get(0))
        .ok();

    if existing.is_some() {
        return Err(AppError::Validation(format!(
            "agent '{}' is already installed", curated.name
        )));
    }

    conn.execute(
        "INSERT INTO agents (id, name, publisher, version, description, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)",
        rusqlite::params![id, curated.name, curated.publisher, curated.version, curated.description, now, now],
    )?;

    // Pre-populate scope grants
    for scope_str in curated.requested_scopes {
        let grant_id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT OR IGNORE INTO agent_scope_grants (id, agent_id, scope, access_mode)
             VALUES (?1, ?2, ?3, 'always')",
            rusqlite::params![grant_id, id, scope_str],
        );
    }

    Ok(Agent {
        id,
        name: curated.name.to_string(),
        publisher: curated.publisher.to_string(),
        version: curated.version.to_string(),
        description: Some(curated.description.to_string()),
        manifest_url: None,
        icon_url: None,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}
