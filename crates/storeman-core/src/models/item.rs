use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemType {
    Consumable,
    NonConsumable,
    Serialized,
    Controlled,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Consumable => write!(f, "Consumable"),
            ItemType::NonConsumable => write!(f, "Non-Consumable"),
            ItemType::Serialized => write!(f, "Serialized"),
            ItemType::Controlled => write!(f, "Controlled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ControlledCategory {
    None,
    ControlledGoods,
    ITAR,
    COMSECMetadata,
    Custom(String),
}

impl std::fmt::Display for ControlledCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlledCategory::None => write!(f, "None"),
            ControlledCategory::ControlledGoods => write!(f, "Controlled Goods"),
            ControlledCategory::ITAR => write!(f, "ITAR"),
            ControlledCategory::COMSECMetadata => write!(f, "COMSEC Metadata"),
            ControlledCategory::Custom(s) => write!(f, "Custom: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub barcode: Option<String>,
    pub nsn: Option<String>,
    pub part_number: Option<String>,
    pub description: String,
    pub category: String,
    pub item_type: ItemType,
    pub unit_of_issue: String,
    pub controlled_category: ControlledCategory,
    pub reorder_point: Option<i64>,
    pub shelf_life_days: Option<i64>,
    pub notes: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Item {
    pub fn is_controlled(&self) -> bool {
        !matches!(self.controlled_category, ControlledCategory::None)
    }
}

pub fn parse_controlled_category(s: &str) -> ControlledCategory {
    match s {
        "None" => ControlledCategory::None,
        "Controlled Goods" => ControlledCategory::ControlledGoods,
        "ITAR" => ControlledCategory::ITAR,
        "COMSEC Metadata" => ControlledCategory::COMSECMetadata,
        other => ControlledCategory::Custom(other.to_string()),
    }
}

pub fn parse_item_type(s: &str) -> ItemType {
    match s {
        "Consumable" => ItemType::Consumable,
        "Non-Consumable" => ItemType::NonConsumable,
        "Serialized" => ItemType::Serialized,
        "Controlled" => ItemType::Controlled,
        _ => ItemType::NonConsumable,
    }
}
