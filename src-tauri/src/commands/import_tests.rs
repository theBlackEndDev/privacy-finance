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

    // Keep ColumnMapping in scope to suppress unused-import warning
    fn _use_mapping() -> ColumnMapping {
        ColumnMapping { csv_column: "x".into(), field: "y".into() }
    }
}
