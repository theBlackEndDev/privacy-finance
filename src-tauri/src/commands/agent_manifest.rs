use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::agent::Agent;
use crate::models::scope::Scope;

/// Agent manifest as fetched from a manifest URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentManifest {
    pub agent_id: Option<String>,
    pub name: String,
    pub publisher: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub requested_scopes: Vec<String>,
}

/// Result of parsing a manifest — includes plain-English scope translations.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedManifest {
    pub manifest: AgentManifest,
    pub scope_descriptions: Vec<ScopeDescription>,
    pub valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeDescription {
    pub scope: String,
    pub description: String,
}

/// Parse a manifest JSON string (for when the user pastes it or it's fetched).
#[tauri::command]
pub fn parse_agent_manifest(manifest_json: String) -> AppResult<ParsedManifest> {
    let manifest: AgentManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| AppError::Validation(format!("invalid manifest JSON: {e}")))?;

    validate_manifest(&manifest)
}

/// Register an agent from a parsed manifest — creates the agent in pending state
/// and pre-populates scope grants from the manifest's requested_scopes.
#[tauri::command]
pub fn register_from_manifest(
    manifest_json: String,
    state: State<DbState>,
) -> AppResult<Agent> {
    let manifest: AgentManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| AppError::Validation(format!("invalid manifest JSON: {e}")))?;

    let parsed = validate_manifest(&manifest)?;
    if !parsed.valid {
        return Err(AppError::Validation(format!(
            "manifest has errors: {}",
            parsed.errors.join(", ")
        )));
    }

    let conn = state.write_conn.lock().unwrap();
    let id = manifest.agent_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = now_utc();
    let version = manifest.version.unwrap_or_else(|| "1.0".to_string());

    conn.execute(
        "INSERT INTO agents (id, name, publisher, version, description, icon_url, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8)",
        rusqlite::params![
            id, manifest.name, manifest.publisher, version,
            manifest.description, manifest.icon_url, now, now
        ],
    )?;

    // Pre-populate scope grants from manifest (all set to 'always' by default)
    for scope_str in &manifest.requested_scopes {
        if Scope::parse(scope_str).is_ok() {
            let grant_id = Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT OR IGNORE INTO agent_scope_grants (id, agent_id, scope, access_mode)
                 VALUES (?1, ?2, ?3, 'always')",
                rusqlite::params![grant_id, id, scope_str],
            );
        }
    }

    Ok(Agent {
        id,
        name: manifest.name,
        publisher: manifest.publisher,
        version,
        description: manifest.description,
        manifest_url: None,
        icon_url: manifest.icon_url,
        status: "pending".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })
}

fn validate_manifest(manifest: &AgentManifest) -> AppResult<ParsedManifest> {
    let mut errors = Vec::new();
    let mut scope_descriptions = Vec::new();

    if manifest.name.trim().is_empty() {
        errors.push("name is required".to_string());
    }
    if manifest.publisher.trim().is_empty() {
        errors.push("publisher is required".to_string());
    }

    for scope_str in &manifest.requested_scopes {
        match Scope::parse(scope_str) {
            Ok(scope) => {
                scope_descriptions.push(ScopeDescription {
                    scope: scope_str.clone(),
                    description: scope.to_human_readable(),
                });
            }
            Err(_) => {
                errors.push(format!("invalid scope: {scope_str}"));
            }
        }
    }

    Ok(ParsedManifest {
        manifest: manifest.clone(),
        scope_descriptions,
        valid: errors.is_empty(),
        errors,
    })
}
