use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::agent::AuditEntry;

#[tauri::command]
pub fn get_audit_log(
    agent_id: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<DbState>,
) -> AppResult<Vec<AuditEntry>> {
    let conn = state.read_conn.lock().unwrap();
    let limit = limit.unwrap_or(100).min(1000);
    let offset = offset.unwrap_or(0);

    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(ref aid) = agent_id {
        (
            "SELECT id, agent_id, scope_requested, scope_granted, status, result_summary, created_at
             FROM agent_audit_log WHERE agent_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
                .to_string(),
            vec![
                Box::new(aid.clone()) as Box<dyn rusqlite::types::ToSql>,
                Box::new(limit),
                Box::new(offset),
            ],
        )
    } else {
        (
            "SELECT id, agent_id, scope_requested, scope_granted, status, result_summary, created_at
             FROM agent_audit_log ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
                .to_string(),
            vec![
                Box::new(limit) as Box<dyn rusqlite::types::ToSql>,
                Box::new(offset),
            ],
        )
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let entries = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                scope_requested: row.get(2)?,
                scope_granted: row.get(3)?,
                status: row.get(4)?,
                result_summary: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}
