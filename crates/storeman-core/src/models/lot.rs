use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotRecord {
    pub id: Uuid,
    pub item_id: Uuid,
    pub lot_number: String,
    pub expiry_date: Option<NaiveDate>,
    pub quantity: i64,
    pub location_id: Option<Uuid>,
    pub received_at: DateTime<Utc>,
    pub notes: String,
    pub active: bool,
}

impl LotRecord {
    pub fn days_until_expiry(&self) -> Option<i64> {
        self.expiry_date.map(|exp| {
            let today = chrono::Utc::now().date_naive();
            (exp - today).num_days()
        })
    }
    pub fn is_expired(&self) -> bool {
        self.days_until_expiry().map(|d| d < 0).unwrap_or(false)
    }
    pub fn is_expiring_soon(&self, days: i64) -> bool {
        self.days_until_expiry().map(|d| d >= 0 && d <= days).unwrap_or(false)
    }
}
