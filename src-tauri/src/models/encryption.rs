use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptionStatus {
    pub is_encrypted: bool,
}
