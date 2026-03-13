use chrono::Utc;

/// Returns the current UTC time as an RFC3339 string.
/// e.g. "2026-03-13T12:00:00.000000000Z"
/// All created_at / updated_at fields must use this function.
pub fn now_utc() -> String {
    Utc::now().to_rfc3339()
}
