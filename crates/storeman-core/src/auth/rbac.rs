use crate::models::{User, Role};
use crate::error::{Result, StoremanError};

pub fn require_can_transact(user: &User) -> Result<()> {
    if user.role.can_transact() { Ok(()) } else {
        Err(StoremanError::Unauthorized(format!("{} cannot perform transactions", user.role)))
    }
}

pub fn require_can_approve(user: &User) -> Result<()> {
    if user.role.can_approve() { Ok(()) } else {
        Err(StoremanError::Unauthorized(format!("{} cannot approve transactions", user.role)))
    }
}

pub fn require_can_export(user: &User) -> Result<()> {
    if user.role.can_export() { Ok(()) } else {
        Err(StoremanError::Unauthorized(format!("{} cannot export reports", user.role)))
    }
}

pub fn require_can_admin(user: &User) -> Result<()> {
    if user.role.can_admin() { Ok(()) } else {
        Err(StoremanError::Unauthorized(format!("{} does not have admin access", user.role)))
    }
}

pub fn require_can_view_controlled(user: &User) -> Result<()> {
    if user.role.can_view_controlled() { Ok(()) } else {
        Err(StoremanError::Unauthorized(format!("{} cannot view controlled items", user.role)))
    }
}
