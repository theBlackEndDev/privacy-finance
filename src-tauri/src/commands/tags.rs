use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::tag::{CreateTagInput, Tag};

fn row_to_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: row.get(0)?,
        name: row.get(1)?,
        color: row.get(2)?,
        created_at: row.get(3)?,
    })
}

#[tauri::command]
pub fn create_tag(
    input: CreateTagInput,
    state: State<DbState>,
) -> AppResult<Tag> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();

    conn.execute(
        "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, input.name, input.color, now],
    )?;

    Ok(Tag {
        id,
        name: input.name,
        color: input.color,
        created_at: now,
    })
}

#[tauri::command]
pub fn get_all_tags(state: State<DbState>) -> AppResult<Vec<Tag>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, color, created_at FROM tags ORDER BY name ASC"
    )?;
    let tags = stmt
        .query_map([], row_to_tag)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

#[tauri::command]
pub fn update_tag(
    id: String,
    name: Option<String>,
    color: Option<String>,
    state: State<DbState>,
) -> AppResult<Tag> {
    let conn = state.write_conn.lock().unwrap();

    let current = conn
        .query_row(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
            [&id],
            row_to_tag,
        )
        .map_err(|_| AppError::NotFound(format!("tag {id}")))?;

    let new_name = name.unwrap_or(current.name);
    let new_color = color.or(current.color);

    conn.execute(
        "UPDATE tags SET name = ?1, color = ?2 WHERE id = ?3",
        rusqlite::params![new_name, new_color, id],
    )?;

    Ok(Tag {
        id,
        name: new_name,
        color: new_color,
        created_at: current.created_at,
    })
}

#[tauri::command]
pub fn delete_tag(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let rows = conn.execute("DELETE FROM tags WHERE id = ?1", [&id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("tag {id}")));
    }
    Ok(())
}

#[tauri::command]
pub fn tag_transaction(
    transaction_id: String,
    tag_id: String,
    state: State<DbState>,
) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();
    conn.execute(
        "INSERT OR IGNORE INTO transaction_tags (transaction_id, tag_id, created_at)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![transaction_id, tag_id, now],
    )?;
    Ok(())
}

#[tauri::command]
pub fn untag_transaction(
    transaction_id: String,
    tag_id: String,
    state: State<DbState>,
) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    conn.execute(
        "DELETE FROM transaction_tags WHERE transaction_id = ?1 AND tag_id = ?2",
        rusqlite::params![transaction_id, tag_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::{run_migrations, seed_test_data};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    fn get_account_id(conn: &rusqlite::Connection) -> String {
        conn.query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0)).unwrap()
    }

    fn create_test_transaction(conn: &rusqlite::Connection, account_id: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-15', 5000, 'expense', 'Store', '', 0, ?3, ?3)",
            rusqlite::params![id, account_id, now],
        ).unwrap();
        id
    }

    #[test]
    fn test_tag_crud() {
        let conn = setup();
        let now = now_utc();

        // Create
        let tag_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, 'Vacation', '#00ff00', ?2)",
            rusqlite::params![tag_id, now],
        ).unwrap();

        // Read
        let tag = conn.query_row(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
            [&tag_id], row_to_tag,
        ).unwrap();
        assert_eq!(tag.name, "Vacation");
        assert_eq!(tag.color, Some("#00ff00".to_string()));

        // Update
        conn.execute("UPDATE tags SET name = 'Holiday' WHERE id = ?1", [&tag_id]).unwrap();
        let updated = conn.query_row(
            "SELECT id, name, color, created_at FROM tags WHERE id = ?1",
            [&tag_id], row_to_tag,
        ).unwrap();
        assert_eq!(updated.name, "Holiday");

        // Delete
        conn.execute("DELETE FROM tags WHERE id = ?1", [&tag_id]).unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tags WHERE id = ?1",
            [&tag_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_tag_untag_transaction() {
        let conn = setup();
        let now = now_utc();
        let account_id = get_account_id(&conn);
        let txn_id = create_test_transaction(&conn, &account_id);

        let tag_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, 'Business', '#0000ff', ?2)",
            rusqlite::params![tag_id, now],
        ).unwrap();

        // Tag
        conn.execute(
            "INSERT INTO transaction_tags (transaction_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![txn_id, tag_id, now],
        ).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transaction_tags WHERE transaction_id = ?1 AND tag_id = ?2",
            rusqlite::params![txn_id, tag_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);

        // Untag
        conn.execute(
            "DELETE FROM transaction_tags WHERE transaction_id = ?1 AND tag_id = ?2",
            rusqlite::params![txn_id, tag_id],
        ).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transaction_tags WHERE transaction_id = ?1",
            [&txn_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_query_transactions_by_tag() {
        let conn = setup();
        let now = now_utc();
        let account_id = get_account_id(&conn);
        let txn1 = create_test_transaction(&conn, &account_id);
        create_test_transaction(&conn, &account_id);

        let tag_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, 'Travel', NULL, ?2)",
            rusqlite::params![tag_id, now],
        ).unwrap();

        // Tag only txn1
        conn.execute(
            "INSERT INTO transaction_tags (transaction_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![txn1, tag_id, now],
        ).unwrap();

        // Query by tag
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions t
             JOIN transaction_tags tt ON t.id = tt.transaction_id
             WHERE tt.tag_id = ?1",
            [&tag_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_delete_tag_cascades_junction() {
        let conn = setup();
        let now = now_utc();
        let account_id = get_account_id(&conn);
        let txn_id = create_test_transaction(&conn, &account_id);

        let tag_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO tags (id, name, color, created_at) VALUES (?1, 'Temp', NULL, ?2)",
            rusqlite::params![tag_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO transaction_tags (transaction_id, tag_id, created_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![txn_id, tag_id, now],
        ).unwrap();

        // Delete tag — should cascade
        conn.execute("DELETE FROM tags WHERE id = ?1", [&tag_id]).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transaction_tags WHERE tag_id = ?1",
            [&tag_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }
}
