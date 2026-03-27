use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub description: Option<String>,
    pub manifest_url: Option<String>,
    pub icon_url: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentInput {
    pub name: String,
    pub publisher: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub manifest_url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentInput {
    pub name: Option<String>,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub manifest_url: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentKey {
    pub id: String,
    pub agent_id: String,
    pub scopes: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeGrant {
    pub id: String,
    pub agent_id: String,
    pub scope: String,
    pub access_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: String,
    pub agent_id: String,
    pub scope_requested: String,
    pub scope_granted: String,
    pub status: String,
    pub result_summary: Option<String>,
    pub created_at: String,
}
