use tauri::State;
use uuid::Uuid;
use chrono::{Datelike, NaiveDate};
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::recurring::{
    RecurringTransaction, CreateRecurringInput, UpdateRecurringInput,
    DueRecurringItem, GenerateResult,
};

fn row_to_recurring(row: &rusqlite::Row) -> rusqlite::Result<RecurringTransaction> {
    Ok(RecurringTransaction {
        id: row.get(0)?,
        account_id: row.get(1)?,
        payee: row.get(2)?,
        description: row.get(3)?,
        amount: row.get(4)?,
        kind: row.get(5)?,
        category_id: row.get(6)?,
        transfer_to_account_id: row.get(7)?,
        frequency: row.get(8)?,
        start_date: row.get(9)?,
        end_date: row.get(10)?,
        next_date: row.get(11)?,
        is_active: row.get::<_, i64>(12)? != 0,
        auto_post: row.get::<_, i64>(13)? != 0,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}

const SELECT_COLS: &str =
    "id, account_id, payee, description, amount, kind, category_id, \
     transfer_to_account_id, frequency, start_date, end_date, next_date, \
     is_active, auto_post, created_at, updated_at";

#[tauri::command]
pub fn get_all_recurring(state: State<DbState>) -> AppResult<Vec<RecurringTransaction>> {
    let conn = state.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLS} FROM recurring_transactions ORDER BY next_date ASC"
    ))?;
    let items = stmt.query_map([], |row| row_to_recurring(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[tauri::command]
pub fn get_recurring(id: String, state: State<DbState>) -> AppResult<RecurringTransaction> {
    let conn = state.read_conn.lock().unwrap();
    conn.query_row(
        &format!("SELECT {SELECT_COLS} FROM recurring_transactions WHERE id = ?1"),
        [&id],
        |row| row_to_recurring(row),
    )
    .map_err(|_| AppError::NotFound(format!("recurring transaction {id}")))
}

#[tauri::command]
pub fn create_recurring(input: CreateRecurringInput, state: State<DbState>) -> AppResult<RecurringTransaction> {
    let conn = state.write_conn.lock().unwrap();
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let payee = input.payee.unwrap_or_default();
    let description = input.description.unwrap_or_default();
    let auto_post = input.auto_post.unwrap_or(false);

    validate_frequency(&input.frequency)?;
    validate_date(&input.start_date, "start_date")?;
    if let Some(ref ed) = input.end_date {
        validate_date(ed, "end_date")?;
    }

    // next_date starts at start_date
    let next_date = &input.start_date;

    conn.execute(
        "INSERT INTO recurring_transactions
         (id, account_id, payee, description, amount, kind, category_id,
          transfer_to_account_id, frequency, start_date, end_date, next_date,
          is_active, auto_post, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1, ?13, ?14, ?15)",
        rusqlite::params![
            id, input.account_id, payee, description, input.amount, input.kind,
            input.category_id, input.transfer_to_account_id, input.frequency,
            input.start_date, input.end_date, next_date,
            auto_post as i64, now, now,
        ],
    )?;

    Ok(RecurringTransaction {
        id,
        account_id: input.account_id,
        payee,
        description,
        amount: input.amount,
        kind: input.kind,
        category_id: input.category_id,
        transfer_to_account_id: input.transfer_to_account_id,
        frequency: input.frequency,
        start_date: input.start_date.clone(),
        end_date: input.end_date,
        next_date: input.start_date,
        is_active: true,
        auto_post,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_recurring(id: String, input: UpdateRecurringInput, state: State<DbState>) -> AppResult<RecurringTransaction> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    let current = conn.query_row(
        "SELECT account_id, payee, description, amount, kind, category_id,
                transfer_to_account_id, frequency, start_date, end_date, next_date,
                is_active, auto_post
         FROM recurring_transactions WHERE id = ?1",
        [&id],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, Option<String>>(9)?,
            row.get::<_, String>(10)?,
            row.get::<_, i64>(11)?,
            row.get::<_, i64>(12)?,
        )),
    ).map_err(|_| AppError::NotFound(format!("recurring transaction {id}")))?;

    let account_id = input.account_id.unwrap_or(current.0);
    let payee = input.payee.unwrap_or(current.1);
    let description = input.description.unwrap_or(current.2);
    let amount = input.amount.unwrap_or(current.3);
    let kind = input.kind.unwrap_or(current.4);
    let category_id = if input.category_id.is_some() { input.category_id } else { current.5 };
    let transfer_to = if input.transfer_to_account_id.is_some() { input.transfer_to_account_id } else { current.6 };
    let frequency = input.frequency.unwrap_or(current.7);
    let start_date = input.start_date.unwrap_or(current.8);
    let end_date = if input.end_date.is_some() { input.end_date } else { current.9 };
    let next_date = input.next_date.unwrap_or(current.10);
    let is_active = input.is_active.map(|b| b as i64).unwrap_or(current.11);
    let auto_post = input.auto_post.map(|b| b as i64).unwrap_or(current.12);

    validate_frequency(&frequency)?;

    conn.execute(
        "UPDATE recurring_transactions SET
         account_id=?1, payee=?2, description=?3, amount=?4, kind=?5,
         category_id=?6, transfer_to_account_id=?7, frequency=?8,
         start_date=?9, end_date=?10, next_date=?11, is_active=?12,
         auto_post=?13, updated_at=?14
         WHERE id=?15",
        rusqlite::params![
            account_id, payee, description, amount, kind,
            category_id, transfer_to, frequency,
            start_date, end_date, next_date, is_active,
            auto_post, now, id,
        ],
    )?;

    drop(conn);

    let read = state.read_conn.lock().unwrap();
    let created_at: String = read.query_row(
        "SELECT created_at FROM recurring_transactions WHERE id = ?1", [&id], |r| r.get(0)
    ).map_err(|_| AppError::NotFound(format!("recurring transaction {id}")))?;

    Ok(RecurringTransaction {
        id,
        account_id,
        payee,
        description,
        amount,
        kind,
        category_id,
        transfer_to_account_id: transfer_to,
        frequency,
        start_date,
        end_date,
        next_date,
        is_active: is_active != 0,
        auto_post: auto_post != 0,
        created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_recurring(id: String, state: State<DbState>) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let affected = conn.execute("DELETE FROM recurring_transactions WHERE id = ?1", [&id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("recurring transaction {id}")));
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_recurring_active(id: String, state: State<DbState>) -> AppResult<RecurringTransaction> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    let current_active: i64 = conn.query_row(
        "SELECT is_active FROM recurring_transactions WHERE id = ?1",
        [&id],
        |r| r.get(0),
    ).map_err(|_| AppError::NotFound(format!("recurring transaction {id}")))?;

    let new_active = if current_active != 0 { 0i64 } else { 1i64 };

    conn.execute(
        "UPDATE recurring_transactions SET is_active = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_active, now, id],
    )?;

    drop(conn);

    let read = state.read_conn.lock().unwrap();
    read.query_row(
        &format!("SELECT {SELECT_COLS} FROM recurring_transactions WHERE id = ?1"),
        [&id],
        |row| row_to_recurring(row),
    )
    .map_err(|_| AppError::NotFound(format!("recurring transaction {id}")))
}

#[tauri::command]
pub fn check_due_recurring(state: State<DbState>) -> AppResult<Vec<DueRecurringItem>> {
    let conn = state.read_conn.lock().unwrap();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLS} FROM recurring_transactions
         WHERE is_active = 1 AND next_date <= ?1
         AND (end_date IS NULL OR end_date >= next_date)
         ORDER BY next_date ASC"
    ))?;

    let items = stmt.query_map([&today], |row| row_to_recurring(row))?
        .collect::<Result<Vec<_>, _>>()?;

    let mut due_items = Vec::new();
    for item in items {
        let next = NaiveDate::parse_from_str(&item.next_date, "%Y-%m-%d")
            .map_err(|e| AppError::Validation(format!("Invalid next_date: {e}")))?;
        let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d")
            .map_err(|e| AppError::Validation(format!("Invalid today date: {e}")))?;

        let mut count = 0i64;
        let mut cursor = next;
        while cursor <= today_date {
            let end_ok = item.end_date.as_ref().map_or(true, |ed| {
                NaiveDate::parse_from_str(ed, "%Y-%m-%d")
                    .map_or(false, |end| cursor <= end)
            });
            if !end_ok { break; }
            count += 1;
            cursor = calculate_next_date_from(cursor, &item.frequency);
        }

        if count > 0 {
            due_items.push(DueRecurringItem {
                recurring: item,
                missed_count: count,
            });
        }
    }

    Ok(due_items)
}

#[tauri::command]
pub fn generate_recurring_transactions(state: State<DbState>) -> AppResult<GenerateResult> {
    let conn = state.write_conn.lock().unwrap();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let today_date = NaiveDate::parse_from_str(&today, "%Y-%m-%d")
        .map_err(|e| AppError::Validation(format!("Invalid today date: {e}")))?;

    let mut stmt = conn.prepare(&format!(
        "SELECT {SELECT_COLS} FROM recurring_transactions
         WHERE is_active = 1 AND next_date <= ?1
         AND (end_date IS NULL OR end_date >= next_date)
         ORDER BY next_date ASC"
    ))?;

    let items = stmt.query_map([&today], |row| row_to_recurring(row))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    let now = now_utc();
    let mut total_generated = 0i64;
    let mut items_processed = 0i64;

    // unchecked_transaction needed because we already hold the conn lock
    // and the standard transaction() requires exclusive &mut which conflicts.
    // This is safe because write_conn is Mutex-protected — only one caller at a time.
    let tx = conn.unchecked_transaction()?;

    for item in &items {
        let mut cursor = NaiveDate::parse_from_str(&item.next_date, "%Y-%m-%d")
            .map_err(|e| AppError::Validation(format!("Invalid next_date: {e}")))?;

        while cursor <= today_date {
            let end_ok = item.end_date.as_ref().map_or(true, |ed| {
                NaiveDate::parse_from_str(ed, "%Y-%m-%d")
                    .map_or(false, |end| cursor <= end)
            });
            if !end_ok { break; }

            let tx_id = Uuid::new_v4().to_string();
            let date_str = cursor.format("%Y-%m-%d").to_string();

            if item.kind == "transfer" {
                if let Some(ref to_account_id) = item.transfer_to_account_id {
                    let group_id = Uuid::new_v4().to_string();

                    // Outgoing leg
                    tx.execute(
                        "INSERT INTO transactions
                         (id, account_id, date, amount, kind, category_id, payee, description,
                          is_cleared, transfer_group_id, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, 'transfer', NULL, ?5, ?6, 0, ?7, ?8, ?9)",
                        rusqlite::params![
                            tx_id, item.account_id, date_str, item.amount,
                            item.payee, item.description, group_id, now, now,
                        ],
                    )?;

                    // Incoming leg
                    let tx_id2 = Uuid::new_v4().to_string();
                    tx.execute(
                        "INSERT INTO transactions
                         (id, account_id, date, amount, kind, category_id, payee, description,
                          is_cleared, transfer_group_id, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, 'transfer', NULL, ?5, ?6, 0, ?7, ?8, ?9)",
                        rusqlite::params![
                            tx_id2, to_account_id, date_str, item.amount,
                            item.payee, item.description, group_id, now, now,
                        ],
                    )?;
                }
            } else {
                tx.execute(
                    "INSERT INTO transactions
                     (id, account_id, date, amount, kind, category_id, payee, description,
                      is_cleared, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10)",
                    rusqlite::params![
                        tx_id, item.account_id, date_str, item.amount, item.kind,
                        item.category_id, item.payee, item.description, now, now,
                    ],
                )?;
            }

            total_generated += 1;
            cursor = calculate_next_date_from(cursor, &item.frequency);
        }

        // Advance next_date
        let new_next = cursor.format("%Y-%m-%d").to_string();
        tx.execute(
            "UPDATE recurring_transactions SET next_date = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![new_next, now, item.id],
        )?;

        // Deactivate if past end_date
        if let Some(ref ed) = item.end_date {
            if let Ok(end) = NaiveDate::parse_from_str(ed, "%Y-%m-%d") {
                if cursor > end {
                    tx.execute(
                        "UPDATE recurring_transactions SET is_active = 0, updated_at = ?1 WHERE id = ?2",
                        rusqlite::params![now, item.id],
                    )?;
                }
            }
        }

        items_processed += 1;
    }

    tx.commit()?;

    Ok(GenerateResult {
        generated_count: total_generated,
        items_processed,
    })
}

fn validate_date(s: &str, field: &str) -> AppResult<()> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| AppError::Validation(format!("{field} must be YYYY-MM-DD, got: {s}")))?;
    Ok(())
}

fn validate_frequency(freq: &str) -> AppResult<()> {
    match freq {
        "daily" | "weekly" | "biweekly" | "monthly" | "quarterly" | "yearly" => Ok(()),
        _ => Err(AppError::Validation(format!("Invalid frequency: {freq}"))),
    }
}

pub fn calculate_next_date_from(current: NaiveDate, frequency: &str) -> NaiveDate {
    match frequency {
        "daily" => current + chrono::Duration::days(1),
        "weekly" => current + chrono::Duration::weeks(1),
        "biweekly" => current + chrono::Duration::weeks(2),
        "monthly" => add_months(current, 1),
        "quarterly" => add_months(current, 3),
        "yearly" => add_months(current, 12),
        _ => current + chrono::Duration::days(1), // fallback
    }
}

fn add_months(date: NaiveDate, months: u32) -> NaiveDate {
    let total_months = date.month0() + months;
    let new_year = date.year() + (total_months / 12) as i32;
    let new_month = (total_months % 12) + 1;

    // Clamp day to last day of target month
    let max_day = days_in_month(new_year, new_month);
    let day = date.day().min(max_day);

    NaiveDate::from_ymd_opt(new_year, new_month, day)
        .unwrap_or(date)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    // Next month's first day minus one day
    if month == 12 {
        31
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
            .pred_opt()
            .unwrap()
            .day()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_next_date_daily() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert_eq!(calculate_next_date_from(d, "daily"), NaiveDate::from_ymd_opt(2026, 3, 16).unwrap());
    }

    #[test]
    fn test_calculate_next_date_weekly() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert_eq!(calculate_next_date_from(d, "weekly"), NaiveDate::from_ymd_opt(2026, 3, 22).unwrap());
    }

    #[test]
    fn test_calculate_next_date_biweekly() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert_eq!(calculate_next_date_from(d, "biweekly"), NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
    }

    #[test]
    fn test_calculate_next_date_monthly() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert_eq!(calculate_next_date_from(d, "monthly"), NaiveDate::from_ymd_opt(2026, 2, 15).unwrap());
    }

    #[test]
    fn test_calculate_next_date_monthly_31st_to_feb() {
        // Jan 31 monthly → Feb 28 (non-leap year 2026)
        let d = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();
        assert_eq!(calculate_next_date_from(d, "monthly"), NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }

    #[test]
    fn test_calculate_next_date_monthly_31st_to_feb_leap() {
        // Jan 31 monthly → Feb 29 (leap year 2028)
        let d = NaiveDate::from_ymd_opt(2028, 1, 31).unwrap();
        assert_eq!(calculate_next_date_from(d, "monthly"), NaiveDate::from_ymd_opt(2028, 2, 29).unwrap());
    }

    #[test]
    fn test_calculate_next_date_quarterly() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert_eq!(calculate_next_date_from(d, "quarterly"), NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
    }

    #[test]
    fn test_calculate_next_date_yearly() {
        let d = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        assert_eq!(calculate_next_date_from(d, "yearly"), NaiveDate::from_ymd_opt(2027, 3, 15).unwrap());
    }

    #[test]
    fn test_calculate_next_date_yearly_leap_day() {
        // Feb 29 yearly → Feb 28 (next non-leap year)
        let d = NaiveDate::from_ymd_opt(2028, 2, 29).unwrap();
        assert_eq!(calculate_next_date_from(d, "yearly"), NaiveDate::from_ymd_opt(2029, 2, 28).unwrap());
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2028, 2), 29);
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }
}
