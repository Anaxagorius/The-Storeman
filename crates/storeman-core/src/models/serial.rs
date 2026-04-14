use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::balance::ConditionCode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialRecord {
    pub id: Uuid,
    pub item_id: Uuid,
    pub serial_number: String,
    pub condition: ConditionCode,
    pub location_id: Option<Uuid>,
    pub custodian_id: Option<Uuid>,
    pub inspection_due: Option<DateTime<Utc>>,
    pub notes: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
