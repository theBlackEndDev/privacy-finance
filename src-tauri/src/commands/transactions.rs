use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::tag::TagBrief;
use crate::models::transaction::{
    CreateTransactionInput, CreateTransferInput, Transaction, TransactionFilters,
    TransactionPage, UpdateTransactionInput,
};

// ── Helper: map a row to Transaction ────────────────────────────────────────

pub(crate) fn row_to_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Transaction> {
    Ok(Transaction {
        id: row.get(0)?,
        account_id: row.get(1)?,
        date: row.get(2)?,
        amount: row.get(3)?,
        kind: row.get(4)?,
        category_id: row.get(5)?,
        payee: row.get(6)?,
        description: row.get(7)?,
        is_cleared: row.get::<_, i64>(8)? != 0,
        is_split: row.get::<_, i64>(9)? != 0,
        transfer_group_id: row.get(10)?,
        import_batch_id: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
        category_name: row.get(14)?,
        category_color: row.get(15)?,
        tags: vec![],
    })
}

pub(crate) const SELECT: &str = "
    SELECT t.id, t.account_id, t.date, t.amount, t.kind,
           t.category_id, t.payee, t.description, t.is_cleared, t.is_split,
           t.transfer_group_id, t.import_batch_id, t.created_at, t.updated_at,
           c.name AS category_name, c.color AS category_color
    FROM transactions t
    LEFT JOIN categories c ON t.category_id = c.id
";

// ── Commands ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn create_transaction(
    input: CreateTransactionInput,
    state: State<DbState>,
) -> AppResult<Transaction> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let payee = input.payee.unwrap_or_default();
    let description = input.description.unwrap_or_default();
    let is_cleared = input.is_cleared.unwrap_or(false) as i64;

    conn.execute(
        "INSERT INTO transactions
         (id, account_id, date, amount, kind, category_id, payee, description,
          is_cleared, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        rusqlite::params![
            id, input.account_id, input.date, input.amount, input.kind,
            input.category_id, payee, description, is_cleared, now, now
        ],
    )?;

    drop(conn);
    get_transaction(id, state)
}

#[tauri::command]
pub fn get_transaction(id: String, state: State<DbState>) -> AppResult<Transaction> {
    let conn = state.read_conn.lock().unwrap();
    let sql = format!("{SELECT} WHERE t.id = ?1");
    conn.query_row(&sql, [&id], row_to_transaction)
        .map_err(|_| AppError::NotFound(format!("transaction {id}")))
}

#[tauri::command]
pub fn update_transaction(
    id: String,
    input: UpdateTransactionInput,
    state: State<DbState>,
) -> AppResult<Transaction> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    let current = conn.query_row(
        "SELECT date, amount, kind, category_id, payee, description, is_cleared
         FROM transactions WHERE id = ?1",
        [&id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        },
    )
    .map_err(|_| AppError::NotFound(format!("transaction {id}")))?;

    let date = input.date.unwrap_or(current.0);
    let amount = input.amount.unwrap_or(current.1);
    let kind = input.kind.unwrap_or(current.2);
    let category_id = if input.category_id.is_some() { input.category_id } else { current.3 };
    let payee = input.payee.unwrap_or(current.4);
    let description = input.description.unwrap_or(current.5);
    let is_cleared = input.is_cleared.map(|b| b as i64).unwrap_or(current.6);

    conn.execute(
        "UPDATE transactions SET date=?1, amount=?2, kind=?3, category_id=?4,
         payee=?5, description=?6, is_cleared=?7, updated_at=?8 WHERE id=?9",
        rusqlite::params![date, amount, kind, category_id, payee, description, is_cleared, now, id],
    )?;

    drop(conn);
    get_transaction(id, state)
}

#[tauri::command]
pub fn delete_transaction(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let rows = conn.execute("DELETE FROM transactions WHERE id = ?1", [&id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("transaction {id}")));
    }
    Ok(())
}

#[tauri::command]
pub fn get_transactions(
    filters: TransactionFilters,
    state: State<DbState>,
) -> AppResult<TransactionPage> {
    let conn = state.read_conn.lock().unwrap();
    let page = filters.page.unwrap_or(0).max(0);
    let page_size = filters.page_size.unwrap_or(50).clamp(1, 200);

    // Build WHERE clauses dynamically
    let mut conditions: Vec<String> = vec![];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(ref account_id) = filters.account_id {
        conditions.push(format!("t.account_id = ?{}", params.len() + 1));
        params.push(Box::new(account_id.clone()));
    }
    if let Some(ref kind) = filters.kind {
        conditions.push(format!("t.kind = ?{}", params.len() + 1));
        params.push(Box::new(kind.clone()));
    }
    if let Some(ref category_id) = filters.category_id {
        conditions.push(format!("t.category_id = ?{}", params.len() + 1));
        params.push(Box::new(category_id.clone()));
    }
    if let Some(ref tag_id) = filters.tag_id {
        conditions.push(format!(
            "t.id IN (SELECT transaction_id FROM transaction_tags WHERE tag_id = ?{})",
            params.len() + 1
        ));
        params.push(Box::new(tag_id.clone()));
    }
    if let Some(ref date_from) = filters.date_from {
        conditions.push(format!("t.date >= ?{}", params.len() + 1));
        params.push(Box::new(date_from.clone()));
    }
    if let Some(ref date_to) = filters.date_to {
        conditions.push(format!("t.date <= ?{}", params.len() + 1));
        params.push(Box::new(date_to.clone()));
    }
    if let Some(ref search) = filters.search {
        let pattern = format!("%{}%", search);
        conditions.push(format!(
            "(t.payee LIKE ?{} OR t.description LIKE ?{})",
            params.len() + 1,
            params.len() + 2
        ));
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count total
    let count_sql = format!(
        "SELECT COUNT(*) FROM transactions t LEFT JOIN categories c ON t.category_id = c.id {where_clause}"
    );
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let total: i64 = conn.query_row(&count_sql, param_refs.as_slice(), |r| r.get(0))?;

    // Fetch page
    let offset = page * page_size;
    let items_sql = format!(
        "{SELECT} {where_clause} ORDER BY t.date DESC, t.created_at DESC LIMIT ?{} OFFSET ?{}",
        params.len() + 1,
        params.len() + 2
    );
    params.push(Box::new(page_size));
    params.push(Box::new(offset));
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&items_sql)?;
    let mut items: Vec<Transaction> = stmt
        .query_map(param_refs.as_slice(), row_to_transaction)?
        .collect::<Result<Vec<_>, _>>()?;

    // Batch-load tags for all fetched transactions
    if !items.is_empty() {
        let ids: Vec<&str> = items.iter().map(|t| t.id.as_str()).collect();
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{i}")).collect();
        let tag_sql = format!(
            "SELECT tt.transaction_id, tg.id, tg.name, tg.color
             FROM transaction_tags tt
             JOIN tags tg ON tt.tag_id = tg.id
             WHERE tt.transaction_id IN ({})",
            placeholders.join(",")
        );
        let tag_params: Vec<&dyn rusqlite::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        let mut tag_stmt = conn.prepare(&tag_sql)?;
        let mut tag_map: HashMap<String, Vec<TagBrief>> = HashMap::new();
        let rows = tag_stmt.query_map(tag_params.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                TagBrief {
                    id: row.get(1)?,
                    name: row.get(2)?,
                    color: row.get(3)?,
                },
            ))
        })?;
        for row in rows {
            let (txn_id, tag) = row?;
            tag_map.entry(txn_id).or_default().push(tag);
        }
        for item in &mut items {
            if let Some(tags) = tag_map.remove(&item.id) {
                item.tags = tags;
            }
        }
    }

    Ok(TransactionPage { items, total, page, page_size })
}

#[tauri::command]
pub fn get_payees(search: String, state: State<DbState>) -> AppResult<Vec<String>> {
    let conn = state.read_conn.lock().unwrap();
    let pattern = format!("%{}%", search);
    let mut stmt = conn.prepare(
        "SELECT DISTINCT payee FROM transactions
         WHERE payee LIKE ?1 AND payee != ''
         ORDER BY payee ASC LIMIT 20",
    )?;
    let payees = stmt
        .query_map([&pattern], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(payees)
}

#[tauri::command]
pub fn create_transfer(
    input: CreateTransferInput,
    state: State<DbState>,
) -> AppResult<(Transaction, Transaction)> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();
    let group_id = Uuid::new_v4().to_string();
    let from_id = Uuid::new_v4().to_string();
    let to_id = Uuid::new_v4().to_string();
    let description = input.description.unwrap_or_default();

    // Validate different accounts
    if input.from_account_id == input.to_account_id {
        return Err(AppError::Validation("Transfer accounts must be different".to_string()));
    }

    conn.execute("BEGIN", [])?;

    let insert = "INSERT INTO transactions
        (id, account_id, date, amount, kind, payee, description,
         is_cleared, transfer_group_id, created_at, updated_at)
        VALUES (?1,?2,?3,?4,'transfer',?5,?6,0,?7,?8,?9)";

    let from_payee = format!("Transfer to account");
    let to_payee = format!("Transfer from account");

    match conn.execute(
        insert,
        rusqlite::params![from_id, input.from_account_id, input.date, input.amount,
            from_payee, description, group_id, now, now],
    ) {
        Ok(_) => {}
        Err(e) => { let _ = conn.execute("ROLLBACK", []); return Err(AppError::Db(e)); }
    }

    match conn.execute(
        insert,
        rusqlite::params![to_id, input.to_account_id, input.date, input.amount,
            to_payee, description, group_id, now, now],
    ) {
        Ok(_) => {}
        Err(e) => { let _ = conn.execute("ROLLBACK", []); return Err(AppError::Db(e)); }
    }

    conn.execute("COMMIT", [])?;
    drop(conn);

    let from_txn = get_transaction(from_id, state.clone())?;
    let to_txn = get_transaction(to_id, state)?;
    Ok((from_txn, to_txn))
}
