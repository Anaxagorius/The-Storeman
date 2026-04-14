use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub warehouse: String,
    pub aisle: String,
    pub rack: String,
    pub bin: String,
    pub description: String,
    pub capacity_note: String,
    pub active: bool,
}

impl Location {
    pub fn display_code(&self) -> String {
        format!("{}-{}-{}-{}", self.warehouse, self.aisle, self.rack, self.bin)
    }
}
