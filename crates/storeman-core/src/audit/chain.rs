use sha2::{Sha256, Digest};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub entry_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Uuid,
    pub user_name: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub details: String,
    pub previous_hash: String,
    pub entry_hash: String,
}

pub struct AuditLog;

impl AuditLog {
    pub fn create_entry(
        user_id: Uuid,
        user_name: &str,
        action: &str,
        entity_type: &str,
        entity_id: Uuid,
        details: &str,
        previous_hash: &str,
    ) -> AuditEntry {
        let entry_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let payload = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            entry_id, timestamp.to_rfc3339(),
            user_id, user_name, action, entity_type, entity_id, details
        );
        let entry_hash = {
            let mut hasher = Sha256::new();
            hasher.update(previous_hash.as_bytes());
            hasher.update(b"|");
            hasher.update(payload.as_bytes());
            hex::encode(hasher.finalize())
        };
        AuditEntry {
            entry_id,
            timestamp,
            user_id,
            user_name: user_name.to_string(),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            details: details.to_string(),
            previous_hash: previous_hash.to_string(),
            entry_hash,
        }
    }
}
