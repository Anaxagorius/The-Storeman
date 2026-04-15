use crate::db::Database;
use crate::models::*;
use crate::error::Result;
use crate::audit::AuditLog;
use crate::auth::require_can_transact;
use uuid::Uuid;
use chrono::Utc;

pub struct TransferParams {
    pub item_id: Uuid,
    pub from_location_id: Uuid,
    pub to_location_id: Uuid,
    pub quantity: i64,
    pub condition: ConditionCode,
    pub notes: String,
}

pub fn transfer(db: &Database, user: &User, params: TransferParams) -> Result<Transaction> {
    require_can_transact(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;

    db.upsert_balance(params.item_id, params.from_location_id, &params.condition, -params.quantity)?;
    db.upsert_balance(params.item_id, params.to_location_id, &params.condition, params.quantity)?;

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::Transfer,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: Some(params.from_location_id),
        to_location_id: Some(params.to_location_id),
        quantity: params.quantity,
        serial_ids: vec![],
        lot_id: None,
        user_id: user.id,
        user_name: user.display_name.clone(),
        approved_by_id: None,
        approved_by_name: None,
        reference: String::new(),
        reason: String::new(),
        notes: params.notes.clone(),
        requires_approval: false,
        approved: true,
        timestamp: Utc::now(),
    };
    db.save_transaction(&tx)?;

    let prev_hash = db.get_last_audit_hash()?;
    let entry = AuditLog::create_entry(
        user.id, &user.display_name,
        "TRANSFER", "Transaction", tx.id,
        &format!("Transferred {} x {} from {} to {}", params.quantity, item.description, params.from_location_id, params.to_location_id),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}
