use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::item::ControlledCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlledProfile {
    pub id: Uuid,
    pub category: ControlledCategory,
    pub requires_dual_approval: bool,
    pub restricted_export: bool,
    pub watermark_text: String,
    pub mandatory_custody_fields: bool,
    pub audit_all_views: bool,
    pub description: String,
}
