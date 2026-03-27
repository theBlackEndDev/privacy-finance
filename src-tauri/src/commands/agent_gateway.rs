use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use axum::{
    Router,
    extract::{Query, State as AxumState},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tower_http::cors::{CorsLayer, AllowOrigin};
use uuid::Uuid;

use crate::error::AppResult;
use crate::helpers::scope_enforcer::{validate_key, enforce_scope, build_where_clause};
use crate::models::scope::Scope;

// ── Gateway shared state ──────────────────────────────────────────

#[derive(Clone)]
struct GatewayAppState {
    read_conn: Arc<Mutex<Connection>>,
    write_conn: Arc<Mutex<Connection>>,
}

// ── Gateway lifecycle ─────────────────────────────────────────────

pub struct GatewayHandle {
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub port: u16,
}

/// Start the local HTTP gateway on localhost:3847.
/// Returns a handle with the shutdown signal and the actual port.
pub fn start_gateway(db_path: PathBuf) -> AppResult<GatewayHandle> {
    let read_conn = Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).map_err(crate::error::AppError::Db)?;
    read_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(crate::error::AppError::Db)?;

    let write_conn = Connection::open(&db_path)
        .map_err(crate::error::AppError::Db)?;
    write_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(crate::error::AppError::Db)?;

    let state = GatewayAppState {
        read_conn: Arc::new(Mutex::new(read_conn)),
        write_conn: Arc::new(Mutex::new(write_conn)),
    };

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let port = 3847u16;

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            origin.as_bytes().starts_with(b"http://localhost")
                || origin.as_bytes().starts_with(b"http://127.0.0.1")
        }))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .route("/v1/agents/register", post(handle_register))
        .route("/v1/agents/{id}/status", get(handle_agent_status))
        .route("/v1/query", get(handle_query))
        .route("/v1/audit", get(handle_audit))
        .route("/v1/health", get(handle_health))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[AgentGateway] Failed to bind to {addr}: {e}");
                return;
            }
        };
        eprintln!("[AgentGateway] Listening on http://{addr}");

        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
                eprintln!("[AgentGateway] Shutting down");
            })
            .await
            .ok();
    });

    Ok(GatewayHandle {
        shutdown_tx: Some(shutdown_tx),
        port,
    })
}

// ── Route handlers ────────────────────────────────────────────────

#[derive(Deserialize)]
struct RegisterInput {
    name: String,
    publisher: String,
    version: Option<String>,
    description: Option<String>,
}

#[derive(Serialize)]
struct RegisterResponse {
    id: String,
    status: String,
    message: String,
}

async fn handle_register(
    AxumState(state): AxumState<GatewayAppState>,
    Json(input): Json<RegisterInput>,
) -> Result<Json<RegisterResponse>, (StatusCode, String)> {
    let id = Uuid::new_v4().to_string();
    let now = crate::helpers::timestamps::now_utc();
    let version = input.version.unwrap_or_else(|| "1.0".to_string());

    let conn = state.write_conn.lock().unwrap();
    conn.execute(
        "INSERT INTO agents (id, name, publisher, version, description, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7)",
        rusqlite::params![id, input.name, input.publisher, version, input.description, now, now],
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RegisterResponse {
        id,
        status: "pending".to_string(),
        message: "Agent registered. Awaiting user approval in PrivacyFinance.".to_string(),
    }))
}

async fn handle_agent_status(
    AxumState(state): AxumState<GatewayAppState>,
    headers: HeaderMap,
    axum::extract::Path(agent_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let raw_key = extract_bearer(&headers)?;
    let conn = state.read_conn.lock().unwrap();
    let validated = validate_key(&raw_key, &conn)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    if validated.agent.id != agent_id {
        return Err((StatusCode::FORBIDDEN, "key does not belong to this agent".to_string()));
    }

    Ok(Json(serde_json::json!({
        "id": validated.agent.id,
        "name": validated.agent.name,
        "status": validated.agent.status,
        "grantedScopes": validated.granted_scopes,
    })))
}

#[derive(Deserialize)]
struct QueryParams {
    scope: String,
    limit: Option<i64>,
}

async fn handle_query(
    AxumState(state): AxumState<GatewayAppState>,
    headers: HeaderMap,
    Query(params): Query<QueryParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let raw_key = extract_bearer(&headers)?;

    let requested_scope = Scope::parse(&params.scope)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let conn = state.read_conn.lock().unwrap();
    let validated = validate_key(&raw_key, &conn)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    // Check scope enforcement
    let granted: Vec<Scope> = validated.granted_scopes.iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();

    if !enforce_scope(&requested_scope, &granted) {
        // Log denial
        log_audit(
            &state.write_conn, &validated.agent.id,
            &params.scope, "", "denied", Some("scope not granted"),
        );
        return Err((StatusCode::FORBIDDEN, format!(
            "scope '{}' is not granted to this agent", params.scope
        )));
    }

    // Build and execute query
    let (where_clause, where_params) = build_where_clause(&requested_scope)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let limit = params.limit.unwrap_or(100).min(1000);
    let sql = format!(
        "SELECT t.id, t.date, t.amount, t.kind, t.payee, t.description, c.name as category
         FROM transactions t
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE {where_clause}
         ORDER BY t.date DESC
         LIMIT {limit}"
    );

    let mut stmt = conn.prepare(&sql)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = where_params.iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();

    let rows: Vec<serde_json::Value> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "date": row.get::<_, String>(1)?,
                "amount": row.get::<_, i64>(2)?,
                "kind": row.get::<_, String>(3)?,
                "payee": row.get::<_, Option<String>>(4)?,
                "description": row.get::<_, Option<String>>(5)?,
                "category": row.get::<_, Option<String>>(6)?,
            }))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM transactions t LEFT JOIN categories c ON t.category_id = c.id WHERE {where_clause}"),
        param_refs.as_slice(),
        |row| row.get(0),
    ).unwrap_or(0);

    // Log success
    log_audit(
        &state.write_conn, &validated.agent.id,
        &params.scope, &params.scope, "approved",
        Some(&format!("{} transactions, total {}", rows.len(), total)),
    );

    Ok(Json(serde_json::json!({
        "total": total,
        "count": rows.len(),
        "transactions": rows,
    })))
}

async fn handle_audit(
    AxumState(state): AxumState<GatewayAppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let raw_key = extract_bearer(&headers)?;
    let conn = state.read_conn.lock().unwrap();
    let validated = validate_key(&raw_key, &conn)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT id, scope_requested, scope_granted, status, result_summary, created_at
         FROM agent_audit_log WHERE agent_id = ?1 ORDER BY created_at DESC LIMIT 100"
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let entries: Vec<serde_json::Value> = stmt
        .query_map(rusqlite::params![validated.agent.id], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "scopeRequested": row.get::<_, String>(1)?,
                "scopeGranted": row.get::<_, String>(2)?,
                "status": row.get::<_, String>(3)?,
                "resultSummary": row.get::<_, Option<String>>(4)?,
                "createdAt": row.get::<_, String>(5)?,
            }))
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(Json(serde_json::json!({ "entries": entries })))
}

async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "privacyfinance-agent-gateway",
        "port": 3847,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────

fn extract_bearer(headers: &HeaderMap) -> Result<String, (StatusCode, String)> {
    let auth = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "missing Authorization header".to_string()))?;

    auth.strip_prefix("Bearer ")
        .map(|s| s.to_string())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "invalid Authorization format".to_string()))
}

fn log_audit(
    write_conn: &Arc<Mutex<Connection>>,
    agent_id: &str,
    scope_requested: &str,
    scope_granted: &str,
    status: &str,
    result_summary: Option<&str>,
) {
    if let Ok(conn) = write_conn.lock() {
        let id = Uuid::new_v4().to_string();
        let now = crate::helpers::timestamps::now_utc();
        let _ = conn.execute(
            "INSERT INTO agent_audit_log (id, agent_id, scope_requested, scope_granted, status, result_summary, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, agent_id, scope_requested, scope_granted, status, result_summary, now],
        );
    }
}

// ── Tauri commands for gateway status ─────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayStatus {
    pub running: bool,
    pub port: u16,
}

use tauri::State;
use crate::db::DbState;

#[tauri::command]
pub fn get_gateway_status(
    state: State<'_, std::sync::Mutex<GatewayHandle>>,
) -> AppResult<GatewayStatus> {
    let handle = state.lock().unwrap();
    Ok(GatewayStatus {
        running: handle.shutdown_tx.is_some(),
        port: handle.port,
    })
}
