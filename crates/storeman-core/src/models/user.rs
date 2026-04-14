use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Storeman,
    CQMS,
    Officer,
    Inspector,
    Admin,
}

impl Role {
    pub fn can_transact(&self) -> bool {
        matches!(self, Role::Storeman | Role::CQMS | Role::Admin)
    }
    pub fn can_approve(&self) -> bool {
        matches!(self, Role::CQMS | Role::Admin)
    }
    pub fn can_export(&self) -> bool {
        matches!(self, Role::CQMS | Role::Officer | Role::Inspector | Role::Admin)
    }
    pub fn can_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }
    pub fn can_view_controlled(&self) -> bool {
        matches!(self, Role::CQMS | Role::Admin | Role::Inspector)
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Storeman => write!(f, "Storeman"),
            Role::CQMS => write!(f, "CQMS"),
            Role::Officer => write!(f, "Officer"),
            Role::Inspector => write!(f, "Inspector"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub rank: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}
