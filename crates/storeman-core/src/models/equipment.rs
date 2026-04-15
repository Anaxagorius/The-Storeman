use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-level category for a piece of equipment, following NATO ACodP-style groupings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EquipmentCategory {
    Weapon,
    Vehicle,
    Optic,
    CES,
    Comms,
    Tool,
    Clothing,
    Ammunition,
    Other,
}

impl std::fmt::Display for EquipmentCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquipmentCategory::Weapon => write!(f, "Weapon"),
            EquipmentCategory::Vehicle => write!(f, "Vehicle"),
            EquipmentCategory::Optic => write!(f, "Optic"),
            EquipmentCategory::CES => write!(f, "CES"),
            EquipmentCategory::Comms => write!(f, "Comms"),
            EquipmentCategory::Tool => write!(f, "Tool"),
            EquipmentCategory::Clothing => write!(f, "Clothing"),
            EquipmentCategory::Ammunition => write!(f, "Ammunition"),
            EquipmentCategory::Other => write!(f, "Other"),
        }
    }
}

pub fn parse_equipment_category(s: &str) -> EquipmentCategory {
    match s {
        "Weapon" => EquipmentCategory::Weapon,
        "Vehicle" => EquipmentCategory::Vehicle,
        "Optic" => EquipmentCategory::Optic,
        "CES" => EquipmentCategory::CES,
        "Comms" => EquipmentCategory::Comms,
        "Tool" => EquipmentCategory::Tool,
        "Clothing" => EquipmentCategory::Clothing,
        "Ammunition" => EquipmentCategory::Ammunition,
        _ => EquipmentCategory::Other,
    }
}

/// CAF service branch associated with a piece of equipment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceBranch {
    Army,
    RCAF,
    RCN,
    Joint,
}

impl std::fmt::Display for ServiceBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceBranch::Army => write!(f, "Army"),
            ServiceBranch::RCAF => write!(f, "RCAF"),
            ServiceBranch::RCN => write!(f, "RCN"),
            ServiceBranch::Joint => write!(f, "Joint"),
        }
    }
}

pub fn parse_service_branch(s: &str) -> ServiceBranch {
    match s {
        "Army" => ServiceBranch::Army,
        "RCAF" => ServiceBranch::RCAF,
        "RCN" => ServiceBranch::RCN,
        _ => ServiceBranch::Joint,
    }
}

/// In-service status of an equipment item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EquipmentStatus {
    InService,
    Limited,
    Legacy,
}

impl std::fmt::Display for EquipmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EquipmentStatus::InService => write!(f, "In Service"),
            EquipmentStatus::Limited => write!(f, "Limited"),
            EquipmentStatus::Legacy => write!(f, "Legacy"),
        }
    }
}

pub fn parse_equipment_status(s: &str) -> EquipmentStatus {
    match s {
        "In Service" => EquipmentStatus::InService,
        "Limited" => EquipmentStatus::Limited,
        "Legacy" => EquipmentStatus::Legacy,
        _ => EquipmentStatus::InService,
    }
}

/// Master Equipment Reference entry — describes what exists in the CAF/NATO universe.
/// This is the static (read-only in normal use) reference layer that unit holdings
/// link to.  It does NOT hold any unit-level quantity or deployment information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentItem {
    pub id: Uuid,
    /// Short, common name used by soldiers (e.g. "C7A2 Rifle").
    pub common_name: String,
    /// Official Canadian designation (e.g. "Rifle, 5.56mm, C7A2").
    pub official_designation: String,
    pub equipment_category: EquipmentCategory,
    /// Optional NATO ACodP-style category code for future codification mapping.
    pub nato_category_code: Option<String>,
    pub manufacturer: Option<String>,
    pub country_of_origin: Option<String>,
    pub service_branch: ServiceBranch,
    pub status: EquipmentStatus,
    /// Year the item entered CAF service.
    pub introduction_year: Option<i32>,
    pub notes: String,
    pub active: bool,
}

/// A specific configuration variant of a master equipment item
/// (e.g. "C7A2 + C79 Optic" as a variant of the C7A2 base item).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentVariant {
    pub id: Uuid,
    pub equipment_id: Uuid,
    /// Human-readable variant name (e.g. "C7A2 + C79A2").
    pub variant_name: String,
    /// Calibre, specification, or key technical parameter.
    pub calibre_or_spec: Option<String>,
    /// Comma-separated list of compatible accessories or sub-components.
    pub compatible_accessories: Option<String>,
    pub notes: String,
}

/// Optional NATO Codification System mapping for an equipment variant.
/// NSN fields are nullable because many CAF-specific items have no public NSN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatoReference {
    pub id: Uuid,
    pub variant_id: Uuid,
    /// NATO Stock Number, where publicly available (e.g. "1005-20-000-1234").
    pub nsn: Option<String>,
    /// NATO reporting name where applicable.
    pub nato_reporting_name: Option<String>,
}
