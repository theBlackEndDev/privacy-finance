use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::exchange_rate::*;

#[tauri::command]
pub fn create_rate(
    db: State<DbState>,
    input: CreateExchangeRateInput,
) -> AppResult<ExchangeRate> {
    let conn = db.write_conn.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO exchange_rates (id, from_currency, to_currency, rate, date, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, input.from_currency, input.to_currency, input.rate, input.date, now],
    )?;

    Ok(ExchangeRate {
        id,
        from_currency: input.from_currency,
        to_currency: input.to_currency,
        rate: input.rate,
        date: input.date,
        created_at: now,
    })
}

#[tauri::command]
pub fn get_rate(
    db: State<DbState>,
    from_currency: String,
    to_currency: String,
    date: String,
) -> AppResult<Option<ExchangeRate>> {
    let conn = db.read_conn.lock().unwrap();

    let result = conn.query_row(
        "SELECT id, from_currency, to_currency, rate, date, created_at
         FROM exchange_rates
         WHERE from_currency = ?1 AND to_currency = ?2
         ORDER BY ABS(julianday(date) - julianday(?3)) ASC
         LIMIT 1",
        rusqlite::params![from_currency, to_currency, date],
        |row| {
            Ok(ExchangeRate {
                id: row.get(0)?,
                from_currency: row.get(1)?,
                to_currency: row.get(2)?,
                rate: row.get(3)?,
                date: row.get(4)?,
                created_at: row.get(5)?,
            })
        },
    );

    match result {
        Ok(rate) => Ok(Some(rate)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub fn get_all_rates(
    db: State<DbState>,
) -> AppResult<Vec<ExchangeRate>> {
    let conn = db.read_conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, from_currency, to_currency, rate, date, created_at
         FROM exchange_rates ORDER BY date DESC",
    )?;

    let rates = stmt
        .query_map([], |row| {
            Ok(ExchangeRate {
                id: row.get(0)?,
                from_currency: row.get(1)?,
                to_currency: row.get(2)?,
                rate: row.get(3)?,
                date: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rates)
}

#[tauri::command]
pub fn delete_rate(
    db: State<DbState>,
    id: String,
) -> AppResult<()> {
    let conn = db.write_conn.lock().unwrap();
    conn.execute("DELETE FROM exchange_rates WHERE id = ?1", [&id])?;
    Ok(())
}

#[tauri::command]
pub fn convert_amount(
    db: State<DbState>,
    amount: i64,
    from_currency: String,
    to_currency: String,
    date: String,
) -> AppResult<i64> {
    let conn = db.read_conn.lock().unwrap();

    if from_currency == to_currency {
        return Ok(amount);
    }

    let rate_result = conn.query_row(
        "SELECT rate FROM exchange_rates
         WHERE from_currency = ?1 AND to_currency = ?2
         ORDER BY ABS(julianday(date) - julianday(?3)) ASC
         LIMIT 1",
        rusqlite::params![from_currency, to_currency, date],
        |row| row.get::<_, f64>(0),
    );

    match rate_result {
        Ok(rate) => Ok((amount as f64 * rate).round() as i64),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(crate::error::AppError::Validation(
                format!("No exchange rate found for {}/{}", from_currency, to_currency),
            ))
        }
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::db::migrations::run_migrations;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn insert_rate(conn: &Connection, from: &str, to: &str, rate: f64, date: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO exchange_rates (id, from_currency, to_currency, rate, date, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, from, to, rate, date, now],
        ).unwrap();
        id
    }

    #[test]
    fn test_exchange_rate_crud() {
        let conn = setup_db();
        let id = insert_rate(&conn, "USD", "EUR", 0.85, "2026-03-01");

        // Get it back
        let rate: f64 = conn.query_row(
            "SELECT rate FROM exchange_rates WHERE id = ?1", [&id], |row| row.get(0)
        ).unwrap();
        assert!((rate - 0.85).abs() < 0.001);

        // Delete
        conn.execute("DELETE FROM exchange_rates WHERE id = ?1", [&id]).unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM exchange_rates WHERE id = ?1", [&id], |row| row.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_nearest_date_lookup() {
        let conn = setup_db();
        insert_rate(&conn, "USD", "EUR", 0.80, "2026-01-01");
        insert_rate(&conn, "USD", "EUR", 0.85, "2026-03-01");
        insert_rate(&conn, "USD", "EUR", 0.90, "2026-06-01");

        // Query for 2026-02-15 should return the 2026-03-01 rate (closest)
        let rate: f64 = conn.query_row(
            "SELECT rate FROM exchange_rates
             WHERE from_currency = 'USD' AND to_currency = 'EUR'
             ORDER BY ABS(julianday(date) - julianday('2026-02-15')) ASC LIMIT 1",
            [], |row| row.get(0)
        ).unwrap();
        // 2026-02-15 is 45 days from Jan 1 and 14 days from Mar 1 — Mar 1 is closer
        assert!((rate - 0.85).abs() < 0.001, "Should return nearest rate (0.85), got {}", rate);
    }

    #[test]
    fn test_convert_amount() {
        let conn = setup_db();
        insert_rate(&conn, "USD", "EUR", 0.85, "2026-03-01");

        // Convert 10000 cents USD to EUR
        let rate: f64 = conn.query_row(
            "SELECT rate FROM exchange_rates
             WHERE from_currency = 'USD' AND to_currency = 'EUR'
             ORDER BY ABS(julianday(date) - julianday('2026-03-01')) ASC LIMIT 1",
            [], |row| row.get(0)
        ).unwrap();

        let converted = (10000_f64 * rate).round() as i64;
        assert_eq!(converted, 8500, "10000 cents * 0.85 = 8500 cents");
    }
}
