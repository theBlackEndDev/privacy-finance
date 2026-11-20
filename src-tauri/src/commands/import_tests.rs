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

    #[test]
    fn parse_amount_review_cases() {
        assert_eq!(parse_amount("$-42.99").unwrap(), -4299);
        assert_eq!(parse_amount("$(42.99)").unwrap(), -4299);
        assert_eq!(parse_amount("42.99-").unwrap(), -4299);
        assert_eq!(parse_amount("−42.99").unwrap(), -4299);
        assert!(parse_amount("2024-01-15").is_err());
        assert!(parse_amount("Check #105").is_err());
        assert_eq!(parse_amount("1.234").unwrap(), 123400);
        assert_eq!(parse_amount("1.234.567").unwrap(), 123456700);
        assert!(parse_amount("1,2,3").is_err());
        assert_eq!(parse_amount("1.234,56").unwrap(), 123456);
        // Leading zero or empty integer part is always a decimal
        assert_eq!(parse_amount("0.125").unwrap(), 13);
        assert_eq!(parse_amount("0.500").unwrap(), 50);
        assert_eq!(parse_amount(".50").unwrap(), 50);
        assert_eq!(parse_amount("$.50").unwrap(), 50);
        assert_eq!(parse_amount("0,125").unwrap(), 13);
        assert_eq!(parse_amount(",50").unwrap(), 50);
        assert_eq!(parse_amount("12.34").unwrap(), 1234);
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

    #[test]
    fn import_decodes_utf16le_bom() {
        use crate::commands::import::import_csv_inner;
        use crate::db::migrations::{run_migrations, seed_test_data};
        use crate::models::import::ImportInput;

        let mut bytes = b"\xFF\xFEd\x00a\x00t\x00e\x00,\x00a\x00m\x00o\x00u\x00n\x00t\x00\n\x00".to_vec();
        for unit in "2024-01-15,-42.99\n".encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let f = write_temp_csv(&bytes);
        let path = f.path().to_str().unwrap().to_string();

        let preview = preview_csv(path.clone()).unwrap();
        assert_eq!(preview.headers, vec!["date", "amount"]);
        assert_eq!(preview.rows, vec![vec!["2024-01-15", "-42.99"]]);

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        let account_id: String = conn
            .query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0))
            .unwrap();

        let map = |c: &str| ColumnMapping { csv_column: c.into(), field: c.into() };
        let input = ImportInput {
            file_path: path,
            account_id,
            mappings: vec![map("date"), map("amount")],
            date_format: "%Y-%m-%d".into(),
            amount_format: "single".into(),
            profile_id: None,
        };
        let result = import_csv_inner(&conn, input).unwrap();
        assert_eq!(result.imported, 1);
        assert!(result.errors.is_empty());

        let (date, amount, kind): (String, i64, String) = conn
            .query_row(
                "SELECT date, amount, kind FROM transactions WHERE import_batch_id = ?1",
                [&result.batch_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(date, "2024-01-15");
        assert_eq!(amount, 4299);
        assert_eq!(kind, "expense");
    }

    #[test]
    fn import_rejects_bad_dates() {
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
        let initial_tx_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))
            .unwrap();

        let map = |c: &str| ColumnMapping { csv_column: c.into(), field: c.into() };

        // Case 1: 2-digit year "01/15/26" with "%m/%d/%Y" -> row error naming 2-digit vs 4-digit year mismatch
        let f_two_digit = write_temp_csv(b"date,payee,amount\n01/15/26,Coffee,-5.00\n");
        let res_two_digit = import_csv_inner(
            &conn,
            ImportInput {
                file_path: f_two_digit.path().to_str().unwrap().to_string(),
                account_id: account_id.clone(),
                mappings: vec![map("date"), map("payee"), map("amount")],
                date_format: "%m/%d/%Y".into(),
                amount_format: "single".into(),
                profile_id: None,
            },
        )
        .unwrap();
        assert_eq!(res_two_digit.imported, 0);
        assert_eq!(res_two_digit.errors.len(), 1);
        assert_eq!(res_two_digit.errors[0].row, 2);
        assert!(
            res_two_digit.errors[0].message.contains("2-digit")
                && res_two_digit.errors[0].message.contains("4-digit"),
            "unexpected error message: {}",
            res_two_digit.errors[0].message
        );

        let after_two_digit_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after_two_digit_count, initial_tx_count);

        // Case 2: Swapped DD/MM file imported as "%m/%d/%Y" -> whole import rejected, nothing inserted
        let f_swapped = write_temp_csv(
            b"date,payee,amount\n05/01/2024,Tesco,-20.00\n15/01/2024,Rent,-800.00\n",
        );
        let res_swapped = import_csv_inner(
            &conn,
            ImportInput {
                file_path: f_swapped.path().to_str().unwrap().to_string(),
                account_id,
                mappings: vec![map("date"), map("payee"), map("amount")],
                date_format: "%m/%d/%Y".into(),
                amount_format: "single".into(),
                profile_id: None,
            },
        );
        assert!(res_swapped.is_err());

        let after_swapped_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM transactions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after_swapped_count, initial_tx_count);
    }

    #[test]
    fn import_keeps_identical_rows_and_dedupes_reimport() {
        use crate::commands::import::import_csv_inner;
        use crate::db::migrations::{run_migrations, seed_test_data};
        use crate::models::import::ImportInput;

        assert_ne!(
            row_hash_pub("acc1", "2024-01-15", -275, "A|B", "C"),
            row_hash_pub("acc1", "2024-01-15", -275, "A", "B|C"),
        );

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        let account_id: String = conn
            .query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0))
            .unwrap();

        let f = write_temp_csv(
            b"date,payee,amount,description\n2024-01-15,MTA,-2.75,Subway\n2024-01-15,MTA,-2.75,Subway\n",
        );
        let path = f.path().to_str().unwrap().to_string();
        let map = |c: &str| ColumnMapping { csv_column: c.into(), field: c.into() };

        let first = import_csv_inner(
            &conn,
            ImportInput {
                file_path: path.clone(),
                account_id: account_id.clone(),
                mappings: vec![map("date"), map("payee"), map("amount"), map("description")],
                date_format: "%Y-%m-%d".into(),
                amount_format: "single".into(),
                profile_id: None,
            },
        )
        .unwrap();
        assert_eq!(first.imported, 2);
        assert_eq!(first.duplicates, 0);
        assert!(first.errors.is_empty());

        let mta_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE payee = 'MTA'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(mta_count, 2);

        let second = import_csv_inner(
            &conn,
            ImportInput {
                file_path: path,
                account_id,
                mappings: vec![map("date"), map("payee"), map("amount"), map("description")],
                date_format: "%Y-%m-%d".into(),
                amount_format: "single".into(),
                profile_id: None,
            },
        )
        .unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.duplicates, 2);
        assert!(second.errors.is_empty());

        let mta_count_after: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM transactions WHERE payee = 'MTA'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(mta_count_after, 2);
    }

    #[test]
    fn reimport_creates_no_empty_batch() {
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

        let f = write_temp_csv(b"date,payee,amount\n2024-01-15,Acme,-50.00\n");
        let path = f.path().to_str().unwrap().to_string();
        let map = |c: &str| ColumnMapping { csv_column: c.into(), field: c.into() };

        let first = import_csv_inner(
            &conn,
            ImportInput {
                file_path: path.clone(),
                account_id: account_id.clone(),
                mappings: vec![map("date"), map("payee"), map("amount")],
                date_format: "%Y-%m-%d".into(),
                amount_format: "single".into(),
                profile_id: None,
            },
        )
        .unwrap();
        assert_eq!(first.imported, 1);
        assert_eq!(first.duplicates, 0);

        let batch_count_after_first: i64 = conn
            .query_row("SELECT COUNT(*) FROM import_batches", [], |r| r.get(0))
            .unwrap();
        assert_eq!(batch_count_after_first, 1);

        let first_batch_rows: i64 = conn
            .query_row(
                "SELECT row_count FROM import_batches WHERE id = ?1",
                [&first.batch_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(first_batch_rows, 1);

        let second = import_csv_inner(
            &conn,
            ImportInput {
                file_path: path,
                account_id,
                mappings: vec![map("date"), map("payee"), map("amount")],
                date_format: "%Y-%m-%d".into(),
                amount_format: "single".into(),
                profile_id: None,
            },
        )
        .unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.duplicates, 1);

        let batch_count_after_second: i64 = conn
            .query_row("SELECT COUNT(*) FROM import_batches", [], |r| r.get(0))
            .unwrap();
        assert_eq!(batch_count_after_second, 1);

        let empty_batches: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM import_batches WHERE row_count = 0",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(empty_batches, 0);
    }

    // Keep ColumnMapping in scope to suppress unused-import warning
    fn _use_mapping() -> ColumnMapping {
        ColumnMapping { csv_column: "x".into(), field: "y".into() }
    }
}
