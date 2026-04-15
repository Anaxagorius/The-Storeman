use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CustodyStatus {
    Active,
    Returned,
    Lost,
    Disposed,
}

impl std::fmt::Display for CustodyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustodyStatus::Active => write!(f, "Active"),
            CustodyStatus::Returned => write!(f, "Returned"),
            CustodyStatus::Lost => write!(f, "Lost"),
            CustodyStatus::Disposed => write!(f, "Disposed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyRecord {
    pub id: Uuid,
    pub item_id: Uuid,
    pub serial_id: Option<Uuid>,
    pub lot_id: Option<Uuid>,
    pub custodian_id: Uuid,
    pub custodian_name: String,
    pub rank: String,
    pub unit: String,
    pub quantity: i64,
    pub issued_at: DateTime<Utc>,
    pub returned_at: Option<DateTime<Utc>>,
    pub transaction_id: Uuid,
    pub status: CustodyStatus,
    pub notes: String,
}
