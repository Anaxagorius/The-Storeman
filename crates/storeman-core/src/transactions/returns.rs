use crate::db::Database;
use crate::models::*;
use crate::error::Result;
use crate::audit::AuditLog;
use crate::auth::require_can_transact;
use uuid::Uuid;
use chrono::Utc;

pub struct ReturnParams {
    pub custody_id: Uuid,
    pub item_id: Uuid,
    pub to_location_id: Uuid,
    pub quantity: i64,
    pub condition: ConditionCode,
    pub notes: String,
}

pub fn process_return(db: &Database, user: &User, params: ReturnParams) -> Result<Transaction> {
    require_can_transact(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;

    db.upsert_balance(params.item_id, params.to_location_id, &params.condition, params.quantity)?;
    db.close_custody(params.custody_id)?;

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::Return,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: None,
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
        "RETURN", "Transaction", tx.id,
        &format!("Returned {} x {}", params.quantity, item.description),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}
