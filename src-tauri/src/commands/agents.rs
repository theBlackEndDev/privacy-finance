use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::agent::{Agent, CreateAgentInput, UpdateAgentInput};

#[tauri::command]
pub fn get_agents(state: State<DbState>) -> AppResult<Vec<Agent>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, publisher, version, description, manifest_url, icon_url, status, created_at, updated_at
         FROM agents ORDER BY created_at DESC"
    )?;
    let agents = stmt
        .query_map([], |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                publisher: row.get(2)?,
                version: row.get(3)?,
                description: row.get(4)?,
                manifest_url: row.get(5)?,
                icon_url: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(agents)
}

#[tauri::command]
pub fn get_agent(id: String, state: State<DbState>) -> AppResult<Agent> {
    let conn = state.read_conn.lock().unwrap();
    conn.query_row(
        "SELECT id, name, publisher, version, description, manifest_url, icon_url, status, created_at, updated_at
         FROM agents WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                publisher: row.get(2)?,
                version: row.get(3)?,
                description: row.get(4)?,
                manifest_url: row.get(5)?,
                icon_url: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(format!("agent {id}")))
}

#[tauri::command]
pub fn create_agent(input: CreateAgentInput, state: State<DbState>) -> AppResult<Agent> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let version = input.version.unwrap_or_else(|| "1.0".to_string());

    conn.execute(
        "INSERT INTO agents (id, name, publisher, version, description, manifest_url, icon_url, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, ?9)",
        rusqlite::params![
            id, input.name, input.publisher, version,
            input.description, input.manifest_url, input.icon_url,
            now, now
        ],
    )?;

    Ok(Agent {
        id,
        name: input.name,
        publisher: input.publisher,
        version,
        description: input.description,
        manifest_url: input.manifest_url,
        icon_url: input.icon_url,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_agent(id: String, input: UpdateAgentInput, state: State<DbState>) -> AppResult<Agent> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    // Build dynamic UPDATE
    let mut sets = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1;

    if let Some(ref name) = input.name {
        sets.push(format!("name = ?{idx}"));
        params.push(Box::new(name.clone()));
        idx += 1;
    }
    if let Some(ref publisher) = input.publisher {
        sets.push(format!("publisher = ?{idx}"));
        params.push(Box::new(publisher.clone()));
        idx += 1;
    }
    if let Some(ref version) = input.version {
        sets.push(format!("version = ?{idx}"));
        params.push(Box::new(version.clone()));
        idx += 1;
    }
    if let Some(ref description) = input.description {
        sets.push(format!("description = ?{idx}"));
        params.push(Box::new(description.clone()));
        idx += 1;
    }
    if let Some(ref manifest_url) = input.manifest_url {
        sets.push(format!("manifest_url = ?{idx}"));
        params.push(Box::new(manifest_url.clone()));
        idx += 1;
    }
    if let Some(ref icon_url) = input.icon_url {
        sets.push(format!("icon_url = ?{idx}"));
        params.push(Box::new(icon_url.clone()));
        idx += 1;
    }

    if sets.is_empty() {
        return get_agent_by_id(&conn, &id);
    }

    sets.push(format!("updated_at = ?{idx}"));
    params.push(Box::new(now));
    idx += 1;

    let sql = format!("UPDATE agents SET {} WHERE id = ?{idx}", sets.join(", "));
    params.push(Box::new(id.clone()));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())?;

    get_agent_by_id(&conn, &id)
}

#[tauri::command]
pub fn update_agent_status(id: String, status: String, state: State<DbState>) -> AppResult<Agent> {
    let valid = ["pending", "approved", "denied", "revoked"];
    if !valid.contains(&status.as_str()) {
        return Err(AppError::Validation(format!(
            "invalid agent status: {status}. Must be one of: {}", valid.join(", ")
        )));
    }

    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();
    let changed = conn.execute(
        "UPDATE agents SET status = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![status, now, id],
    )?;

    if changed == 0 {
        return Err(AppError::NotFound(format!("agent {id}")));
    }

    get_agent_by_id(&conn, &id)
}

#[tauri::command]
pub fn delete_agent(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let changed = conn.execute("DELETE FROM agents WHERE id = ?1", rusqlite::params![id])?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("agent {id}")));
    }
    Ok(())
}

fn get_agent_by_id(conn: &rusqlite::Connection, id: &str) -> AppResult<Agent> {
    conn.query_row(
        "SELECT id, name, publisher, version, description, manifest_url, icon_url, status, created_at, updated_at
         FROM agents WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                publisher: row.get(2)?,
                version: row.get(3)?,
                description: row.get(4)?,
                manifest_url: row.get(5)?,
                icon_url: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(format!("agent {id}")))
}
