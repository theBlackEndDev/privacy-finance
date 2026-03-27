use tauri::State;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use rand::RngCore;
use serde::Serialize;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::agent::AgentKey;

/// Returned exactly once when a key is generated — contains the raw key.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentKeyCreated {
    pub id: String,
    pub agent_id: String,
    pub raw_key: String,
    pub scopes: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

/// Generate a scoped API key for an approved agent.
/// The raw key is returned exactly once and never stored.
#[tauri::command]
pub fn generate_agent_key(
    agent_id: String,
    scopes: Vec<String>,
    expires_in_days: Option<i64>,
    state: State<DbState>,
) -> AppResult<AgentKeyCreated> {
    let conn = state.write_conn.lock().unwrap();

    // Verify agent exists and is approved
    let status: String = conn
        .query_row(
            "SELECT status FROM agents WHERE id = ?1",
            rusqlite::params![agent_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::NotFound(format!("agent {agent_id}")))?;

    if status != "approved" {
        return Err(AppError::AgentAuth(format!(
            "agent {agent_id} is not approved (status: {status})"
        )));
    }

    // Validate all scope strings
    for s in &scopes {
        crate::models::scope::Scope::parse(s)?;
    }

    // Generate random key
    let mut key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key_bytes);
    let raw_key = format!("pf_live_sk_{}", hex::encode(key_bytes));

    // Hash for storage
    let key_hash = hash_key(&raw_key);

    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let scopes_json = serde_json::to_string(&scopes)
        .map_err(|e| AppError::Validation(format!("failed to serialize scopes: {e}")))?;

    let expires_at = expires_in_days.map(|days| {
        let expiry = chrono::Utc::now() + chrono::Duration::days(days);
        expiry.to_rfc3339()
    });

    conn.execute(
        "INSERT INTO agent_keys (id, agent_id, key_hash, scopes, expires_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, agent_id, key_hash, scopes_json, expires_at, now],
    )?;

    Ok(AgentKeyCreated {
        id,
        agent_id,
        raw_key,
        scopes: scopes_json,
        expires_at,
        created_at: now,
    })
}

/// List all keys for an agent (without hashes — those are never exposed).
#[tauri::command]
pub fn get_agent_keys(agent_id: String, state: State<DbState>) -> AppResult<Vec<AgentKey>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, scopes, expires_at, last_used_at, created_at
         FROM agent_keys WHERE agent_id = ?1 ORDER BY created_at DESC"
    )?;
    let keys = stmt
        .query_map(rusqlite::params![agent_id], |row| {
            Ok(AgentKey {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                scopes: row.get(2)?,
                expires_at: row.get(3)?,
                last_used_at: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(keys)
}

/// Revoke (delete) an API key.
#[tauri::command]
pub fn revoke_agent_key(key_id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let changed = conn.execute(
        "DELETE FROM agent_keys WHERE id = ?1",
        rusqlite::params![key_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("agent key {key_id}")));
    }
    Ok(())
}

/// SHA-256 hash a raw key for storage comparison.
pub fn hash_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    hex::encode(hasher.finalize())
}
