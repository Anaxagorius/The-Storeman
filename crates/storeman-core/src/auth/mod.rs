pub mod rbac;
pub use rbac::*;

use argon2::{Argon2, PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::password_hash::SaltString;
use rand::rngs::OsRng;
use crate::error::{Result, StoremanError};
use crate::models::User;
use crate::db::Database;

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| StoremanError::Other(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else { return false; };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

pub fn authenticate(db: &Database, username: &str, password: &str) -> Result<User> {
    let Some((user, hash)) = db.get_user_by_username(username)? else {
        return Err(StoremanError::AuthFailed);
    };
    if !user.active {
        return Err(StoremanError::AuthFailed);
    }
    if !verify_password(password, &hash) {
        return Err(StoremanError::AuthFailed);
    }
    db.update_last_login(user.id)?;
    Ok(user)
}
