#[cfg(test)]
mod tests {
    use crate::commands::import::{parse_amount, preview_csv, row_hash_pub};
    use crate::models::import::ColumnMapping;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ── parse_amount unit tests ──────────────────────────────────────────────

    #[test]
    fn amount_plain() {
        assert_eq!(parse_amount("42.99").unwrap(), 4299);
    }

    #[test]
    fn amount_negative() {
        assert_eq!(parse_amount("-42.99").unwrap(), -4299);
    }

    #[test]
    fn amount_parentheses_negative() {
        assert_eq!(parse_amount("(42.99)").unwrap(), -4299);
    }

    #[test]
    fn amount_us_thousands_separator() {
        assert_eq!(parse_amount("$1,234.56").unwrap(), 123456);
    }

    #[test]
    fn amount_european_format() {
        assert_eq!(parse_amount("1.234,56").unwrap(), 123456);
    }

    #[test]
    fn amount_integer_no_decimals() {
        assert_eq!(parse_amount("100").unwrap(), 10000);
    }

    #[test]
    fn amount_parentheses_with_thousands() {
        assert_eq!(parse_amount("(1,234.56)").unwrap(), -123456);
    }

    // ── row_hash ─────────────────────────────────────────────────────────────

    #[test]
    fn same_row_same_hash() {
        let h1 = row_hash_pub("acc1", "2024-01-01", -5000, "Acme", "");
        let h2 = row_hash_pub("acc1", "2024-01-01", -5000, "Acme", "");
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_date_different_hash() {
        let h1 = row_hash_pub("acc1", "2024-01-01", -5000, "Acme", "");
        let h2 = row_hash_pub("acc1", "2024-01-02", -5000, "Acme", "");
        assert_ne!(h1, h2);
    }

    #[test]
    fn different_account_different_hash() {
        let h1 = row_hash_pub("acc1", "2024-01-01", -5000, "Acme", "");
        let h2 = row_hash_pub("acc2", "2024-01-01", -5000, "Acme", "");
        assert_ne!(h1, h2);
    }

    // ── preview_csv ──────────────────────────────────────────────────────────

    fn write_temp_csv(content: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content).unwrap();
        f
    }

    #[test]
    fn preview_basic_csv() {
        let f = write_temp_csv(b"date,payee,amount\n2024-01-01,Acme,-50.00\n2024-01-02,Shop,-20.00\n");
        let preview = preview_csv(f.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(preview.headers, vec!["date", "payee", "amount"]);
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.delimiter, ',');
    }

    #[test]
    fn preview_strips_utf8_bom() {
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"date,payee,amount\n2024-01-01,Test,-10.00\n");
        let f = write_temp_csv(&content);
        let preview = preview_csv(f.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(preview.headers[0], "date");
    }

    #[test]
    fn preview_semicolon_delimiter() {
        let f = write_temp_csv(b"date;payee;amount\n2024-01-01;Acme;-50.00\n");
        let preview = preview_csv(f.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(preview.delimiter, ';');
        assert_eq!(preview.headers, vec!["date", "payee", "amount"]);
    }

    #[test]
    fn preview_limits_to_10_rows() {
        let mut content = String::from("date,payee,amount\n");
        for i in 0..20 {
            content.push_str(&format!("2024-01-{:02},Shop,-{}.00\n", (i % 28) + 1, i + 1));
        }
        let f = write_temp_csv(content.as_bytes());
        let preview = preview_csv(f.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(preview.rows.len(), 10);
    }

    #[test]
    fn preview_tab_delimiter() {
        let f = write_temp_csv(b"date\tpayee\tamount\n2024-01-01\tAcme\t-50.00\n");
        let preview = preview_csv(f.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(preview.delimiter, '\t');
    }

    // ── import_csv ───────────────────────────────────────────────────────────

    #[test]
    fn import_debit_is_positive_expense() {
        use crate::commands::import::import_csv_inner;
        use crate::db::migrations::{run_migrations, seed_test_data};
        use crate::models::import::ImportInput;

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        let account_id: String = conn
            .query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0))
            .unwrap();

        let f = write_temp_csv(b"date,payee,amount\n2024-01-01,Acme,-50.00\n2024-01-02,Employer,75.00\n");
        let map = |c: &str| ColumnMapping { csv_column: c.into(), field: c.into() };
        let input = ImportInput {
            file_path: f.path().to_str().unwrap().to_string(),
            account_id,
            mappings: vec![map("date"), map("payee"), map("amount")],
            date_format: "%Y-%m-%d".into(),
            amount_format: "single".into(),
            profile_id: None,
        };
        let result = import_csv_inner(&conn, input).unwrap();
        assert_eq!(result.imported, 2);

        let (amount, kind): (i64, String) = conn
            .query_row(
                "SELECT amount, kind FROM transactions WHERE payee = 'Acme'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((amount, kind.as_str()), (5000, "expense"));

        let (amount, kind): (i64, String) = conn
            .query_row(
                "SELECT amount, kind FROM transactions WHERE payee = 'Employer'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((amount, kind.as_str()), (7500, "income"));
    }

    // Keep ColumnMapping in scope to suppress unused-import warning
    fn _use_mapping() -> ColumnMapping {
        ColumnMapping { csv_column: "x".into(), field: "y".into() }
    }
}
