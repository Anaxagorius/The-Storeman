use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Receive,
    Issue,
    Return,
    Transfer,
    Adjust,
    Dispose,
    StocktakeCount,
    StocktakeAdjust,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Receive => write!(f, "Receive"),
            TransactionType::Issue => write!(f, "Issue"),
            TransactionType::Return => write!(f, "Return"),
            TransactionType::Transfer => write!(f, "Transfer"),
            TransactionType::Adjust => write!(f, "Adjust"),
            TransactionType::Dispose => write!(f, "Dispose"),
            TransactionType::StocktakeCount => write!(f, "Stocktake Count"),
            TransactionType::StocktakeAdjust => write!(f, "Stocktake Adjust"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub item_id: Uuid,
    pub item_description: String,
    pub from_location_id: Option<Uuid>,
    pub to_location_id: Option<Uuid>,
    pub quantity: i64,
    pub serial_ids: Vec<Uuid>,
    pub lot_id: Option<Uuid>,
    pub user_id: Uuid,
    pub user_name: String,
    pub approved_by_id: Option<Uuid>,
    pub approved_by_name: Option<String>,
    pub reference: String,
    pub reason: String,
    pub notes: String,
    pub requires_approval: bool,
    pub approved: bool,
    pub timestamp: DateTime<Utc>,
}
