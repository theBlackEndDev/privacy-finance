use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::search::SearchResult;

// ── search_transactions_full: returns full Transaction structs + snippets ──

#[tauri::command]
pub fn search_transactions_full(
    query: String,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<DbState>,
) -> AppResult<Vec<SearchResult>> {
    let q = sanitize_fts_query(&query);
    if q.is_empty() {
        return Ok(vec![]);
    }
    let conn = state.read_conn.lock().unwrap();
    search_transactions_full_inner(&conn, &query, limit, offset)
}

fn search_transactions_full_inner(
    conn: &rusqlite::Connection,
    query: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> AppResult<Vec<SearchResult>> {
    let q = sanitize_fts_query(query);
    if q.is_empty() {
        return Ok(vec![]);
    }
    let limit = limit.unwrap_or(20);
    let offset = offset.unwrap_or(0);

    let mut stmt = conn.prepare(
        "SELECT
            t.id, t.account_id, t.date, t.amount, t.kind,
            t.category_id, t.payee, t.description, t.is_cleared,
            t.transfer_group_id, t.import_batch_id,
            t.created_at, t.updated_at,
            c.name AS category_name, c.color AS category_color,
            snippet(transactions_fts, 1, '<mark>', '</mark>', '…', 32) AS payee_snippet,
            snippet(transactions_fts, 2, '<mark>', '</mark>', '…', 32) AS desc_snippet
         FROM transactions_fts fts
         JOIN transactions t ON t.id = fts.transaction_id
         LEFT JOIN categories c ON c.id = t.category_id
         WHERE transactions_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2 OFFSET ?3"
    )?;

    let results: Vec<SearchResult> = stmt.query_map(
        rusqlite::params![q, limit, offset],
        |row: &rusqlite::Row| {
            let payee_snippet: String = row.get(15)?;
            let desc_snippet: String = row.get(16)?;
            let snippet = if !desc_snippet.is_empty() && desc_snippet.contains("<mark>") {
                desc_snippet
            } else {
                payee_snippet
            };
            Ok(SearchResult {
                id: row.get(0)?,
                account_id: row.get(1)?,
                date: row.get(2)?,
                amount: row.get(3)?,
                kind: row.get(4)?,
                category_id: row.get(5)?,
                payee: row.get(6)?,
                description: row.get(7)?,
                is_cleared: row.get::<_, i64>(8)? != 0,
                transfer_group_id: row.get(9)?,
                import_batch_id: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
                category_name: row.get(13)?,
                category_color: row.get(14)?,
                snippet,
            })
        },
    )?.collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Sanitize FTS5 query to prevent syntax errors.
/// Wraps each word in double quotes for literal matching, appends * for prefix.
fn sanitize_fts_query(input: &str) -> String {
    input
        .split_whitespace()
        .map(|w| w.chars().filter(|&c| c != '"' && c != '\0').collect::<String>())
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{w}\"*"))
        .collect::<Vec<_>>()
        .join(" ")
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::{run_migrations, seed_test_data};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    fn insert_tx(conn: &Connection, id: &str, payee: &str, description: &str) {
        let now = "2026-01-01T00:00:00Z";
        let account_id: String = conn.query_row(
            "SELECT id FROM accounts LIMIT 1", [], |row| row.get(0)
        ).unwrap();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-15', 1000, 'expense', ?3, ?4, 0, ?5, ?5)",
            rusqlite::params![id, account_id, payee, description, now],
        ).unwrap();
    }

    #[test]
    fn test_fts_insert_trigger() {
        let conn = setup_db();
        insert_tx(&conn, "tx-1", "Netflix", "Monthly subscription");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions_fts WHERE transaction_id = 'tx-1'",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fts_update_trigger() {
        let conn = setup_db();
        insert_tx(&conn, "tx-2", "Spotify", "Music streaming");

        conn.execute(
            "UPDATE transactions SET payee = 'Apple Music' WHERE id = 'tx-2'",
            [],
        ).unwrap();

        let payee: String = conn.query_row(
            "SELECT payee FROM transactions_fts WHERE transaction_id = 'tx-2'",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(payee, "Apple Music");
    }

    #[test]
    fn test_fts_delete_trigger() {
        let conn = setup_db();
        insert_tx(&conn, "tx-3", "Hulu", "Streaming service");

        conn.execute("DELETE FROM transactions WHERE id = 'tx-3'", []).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions_fts WHERE transaction_id = 'tx-3'",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fts_search_match() {
        let conn = setup_db();
        insert_tx(&conn, "tx-10", "Netflix", "Monthly subscription");
        insert_tx(&conn, "tx-11", "Grocery Store", "Weekly groceries");

        let mut stmt = conn.prepare(
            "SELECT transaction_id FROM transactions_fts WHERE transactions_fts MATCH '\"Netflix\"*' ORDER BY rank"
        ).unwrap();
        let ids: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok()).collect();

        assert!(ids.contains(&"tx-10".to_string()));
        assert!(!ids.contains(&"tx-11".to_string()));
    }

    #[test]
    fn test_fts_search_description() {
        let conn = setup_db();
        insert_tx(&conn, "tx-20", "Store A", "organic vegetables");
        insert_tx(&conn, "tx-21", "Store B", "electronics repair");

        let mut stmt = conn.prepare(
            "SELECT transaction_id FROM transactions_fts WHERE transactions_fts MATCH '\"organic\"*'"
        ).unwrap();
        let ids: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok()).collect();

        assert!(ids.contains(&"tx-20".to_string()));
        assert!(!ids.contains(&"tx-21".to_string()));
    }

    #[test]
    fn test_fts_ranking() {
        let conn = setup_db();
        insert_tx(&conn, "tx-30", "Netflix", "best streaming");
        insert_tx(&conn, "tx-31", "Cable Co", "cancelled Netflix alternative");

        let mut stmt = conn.prepare(
            "SELECT transaction_id FROM transactions_fts WHERE transactions_fts MATCH '\"Netflix\"*' ORDER BY rank"
        ).unwrap();
        let ids: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok()).collect();

        assert!(ids.len() >= 2);
        assert!(ids.contains(&"tx-30".to_string()));
        assert!(ids.contains(&"tx-31".to_string()));
    }

    #[test]
    fn test_fts_rebuild() {
        let conn = setup_db();
        insert_tx(&conn, "tx-40", "Test Payee", "Test description");

        conn.execute("DELETE FROM transactions_fts", []).unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions_fts", [], |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0);

        crate::db::migrations::rebuild_fts(&conn).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions_fts", [], |row| row.get(0),
        ).unwrap();
        assert!(count >= 1);
    }

    #[test]
    fn test_sanitize_fts_query() {
        assert_eq!(sanitize_fts_query(""), "");
        assert_eq!(sanitize_fts_query("  "), "");
        assert_eq!(sanitize_fts_query("netflix"), "\"netflix\"*");
        assert_eq!(sanitize_fts_query("grocery store"), "\"grocery\"* \"store\"*");
    }

    #[test]
    fn test_fts_multi_word_search() {
        let conn = setup_db();
        insert_tx(&conn, "tx-50", "Whole Foods", "organic grocery shopping");
        insert_tx(&conn, "tx-51", "Best Buy", "electronic parts");

        let q = sanitize_fts_query("organic grocery");
        let mut stmt = conn.prepare(
            "SELECT transaction_id FROM transactions_fts WHERE transactions_fts MATCH ?1"
        ).unwrap();
        let ids: Vec<String> = stmt.query_map(
            rusqlite::params![q], |row| row.get(0)
        ).unwrap().filter_map(|r| r.ok()).collect();

        assert!(ids.contains(&"tx-50".to_string()));
        assert!(!ids.contains(&"tx-51".to_string()));
    }

    #[test]
    fn test_fts_populate_existing() {
        let conn = setup_db();
        let fts_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions_fts", [], |row| row.get(0),
        ).unwrap();
        let tx_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions", [], |row| row.get(0),
        ).unwrap();
        assert_eq!(fts_count, tx_count);
    }

    #[test]
    fn search_handles_quotes_and_nul() {
        let conn = setup_db();
        insert_tx(&conn, "tx-q1", "Whole Foods", "organic groceries");
        insert_tx(&conn, "tx-q2", "foobar", "nul test");

        for input in ["\"", "\"\"", "foo\0bar", "\"Whole\""] {
            let res = search_transactions_full_inner(&conn, input, None, None);
            assert!(res.is_ok(), "expected Ok for input {input:?}, got {res:?}");
        }

        let whole = search_transactions_full_inner(&conn, "\"Whole\"", None, None).unwrap();
        assert!(whole.iter().any(|r| r.payee.contains("Whole")));

        let foobar = search_transactions_full_inner(&conn, "foo\0bar", None, None).unwrap();
        assert!(foobar.iter().any(|r| r.id == "tx-q2"));
    }

    #[test]
    fn search_surfaces_row_errors() {
        let conn = setup_db();
        insert_tx(&conn, "tx-err", "CorruptPayee", "row error test");
        conn.execute(
            "UPDATE transactions SET amount = 'not-an-int' WHERE id = 'tx-err'",
            [],
        ).unwrap();

        let res = search_transactions_full_inner(&conn, "CorruptPayee", None, None);
        assert!(res.is_err(), "expected row error to be surfaced, got {res:?}");
    }
}
