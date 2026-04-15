use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionCode {
    Serviceable,
    Unserviceable,
    Repair,
    Quarantine,
    Condemned,
    Custom(String),
}

impl std::fmt::Display for ConditionCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConditionCode::Serviceable => write!(f, "Serviceable"),
            ConditionCode::Unserviceable => write!(f, "Unserviceable"),
            ConditionCode::Repair => write!(f, "Repair"),
            ConditionCode::Quarantine => write!(f, "Quarantine"),
            ConditionCode::Condemned => write!(f, "Condemned"),
            ConditionCode::Custom(s) => write!(f, "{}", s),
        }
    }
}

pub fn parse_condition_code(s: &str) -> ConditionCode {
    match s {
        "Serviceable" => ConditionCode::Serviceable,
        "Unserviceable" => ConditionCode::Unserviceable,
        "Repair" => ConditionCode::Repair,
        "Quarantine" => ConditionCode::Quarantine,
        "Condemned" => ConditionCode::Condemned,
        other => ConditionCode::Custom(other.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockBalance {
    pub id: Uuid,
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub condition: ConditionCode,
    pub quantity: i64,
    pub updated_at: DateTime<Utc>,
}
