use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::account::{Account, CreateAccountInput, UpdateAccountInput};

#[tauri::command]
pub fn get_accounts(state: State<DbState>) -> AppResult<Vec<Account>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at
         FROM accounts ORDER BY sort_order ASC, name ASC"
    )?;
    let accounts = stmt.query_map([], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            account_type: row.get(2)?,
            currency_code: row.get(3)?,
            opening_balance: row.get(4)?,
            sort_order: row.get(5)?,
            is_archived: row.get::<_, i64>(6)? != 0,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(accounts)
}

#[tauri::command]
pub fn get_account(id: String, state: State<DbState>) -> AppResult<Account> {
    let conn = state.read_conn.lock().unwrap();
    conn.query_row(
        "SELECT id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at
         FROM accounts WHERE id = ?1",
        [&id],
        |row| Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            account_type: row.get(2)?,
            currency_code: row.get(3)?,
            opening_balance: row.get(4)?,
            sort_order: row.get(5)?,
            is_archived: row.get::<_, i64>(6)? != 0,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        }),
    )
    .map_err(|_| AppError::NotFound(format!("account {id}")))
}

#[tauri::command]
pub fn create_account(input: CreateAccountInput, state: State<DbState>) -> AppResult<Account> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let currency = input.currency_code.unwrap_or_else(|| "USD".to_string());
    let balance = input.opening_balance.unwrap_or(0);
    let sort = input.sort_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
        rusqlite::params![id, input.name, input.account_type, currency, balance, sort, now, now],
    )?;

    Ok(Account {
        id,
        name: input.name,
        account_type: input.account_type,
        currency_code: currency,
        opening_balance: balance,
        sort_order: sort,
        is_archived: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_account(id: String, input: UpdateAccountInput, state: State<DbState>) -> AppResult<Account> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    // Fetch current values
    let current = conn.query_row(
        "SELECT name, type, currency_code, opening_balance, sort_order, is_archived FROM accounts WHERE id = ?1",
        [&id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
        )),
    ).map_err(|_| AppError::NotFound(format!("account {id}")))?;

    let name = input.name.unwrap_or(current.0);
    let account_type = input.account_type.unwrap_or(current.1);
    let currency = input.currency_code.unwrap_or(current.2);
    let balance = input.opening_balance.unwrap_or(current.3);
    let sort = input.sort_order.unwrap_or(current.4);
    let archived = input.is_archived.map(|b| b as i64).unwrap_or(current.5);

    conn.execute(
        "UPDATE accounts SET name=?1, type=?2, currency_code=?3, opening_balance=?4, sort_order=?5, is_archived=?6, updated_at=?7 WHERE id=?8",
        rusqlite::params![name, account_type, currency, balance, sort, archived, now, id],
    )?;

    drop(conn);

    // Re-read the created_at from read conn to return complete struct
    let read = state.read_conn.lock().unwrap();
    let created_at: String = read.query_row(
        "SELECT created_at FROM accounts WHERE id = ?1", [&id], |r| r.get(0)
    ).map_err(|_| AppError::NotFound(format!("account {id}")))?;

    Ok(Account {
        id,
        name,
        account_type,
        currency_code: currency,
        opening_balance: balance,
        sort_order: sort,
        is_archived: archived != 0,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_account(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();

    // Check account exists
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM accounts WHERE id = ?1", [&id], |r| r.get::<_, i64>(0)
    ).map(|c| c > 0).unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound(format!("account {id}")));
    }

    // ON DELETE RESTRICT on transactions.account_id means SQLite will error if transactions exist
    conn.execute("DELETE FROM accounts WHERE id = ?1", [&id])
        .map_err(|e| {
            if e.to_string().contains("FOREIGN KEY") {
                AppError::Validation("Cannot delete account with existing transactions".to_string())
            } else {
                AppError::Db(e)
            }
        })?;

    Ok(())
}
