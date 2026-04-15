use crate::db::Database;
use crate::models::*;
use crate::error::Result;
use crate::audit::AuditLog;
use crate::auth::require_can_approve;
use uuid::Uuid;
use chrono::Utc;

pub struct DisposeParams {
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub quantity: i64,
    pub condition: ConditionCode,
    pub reason: String,
    pub notes: String,
}

pub fn dispose(db: &Database, user: &User, params: DisposeParams) -> Result<Transaction> {
    require_can_approve(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;

    db.upsert_balance(params.item_id, params.location_id, &params.condition, -params.quantity)?;

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::Dispose,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: Some(params.location_id),
        to_location_id: None,
        quantity: params.quantity,
        serial_ids: vec![],
        lot_id: None,
        user_id: user.id,
        user_name: user.display_name.clone(),
        approved_by_id: Some(user.id),
        approved_by_name: Some(user.display_name.clone()),
        reference: String::new(),
        reason: params.reason.clone(),
        notes: params.notes.clone(),
        requires_approval: true,
        approved: true,
        timestamp: Utc::now(),
    };
    db.save_transaction(&tx)?;

    let prev_hash = db.get_last_audit_hash()?;
    let entry = AuditLog::create_entry(
        user.id, &user.display_name,
        "DISPOSE", "Transaction", tx.id,
        &format!("Disposed {} x {}: {}", params.quantity, item.description, params.reason),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}
