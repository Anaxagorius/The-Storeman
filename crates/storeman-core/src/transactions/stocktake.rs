use crate::db::Database;
use crate::models::*;
use crate::error::Result;
use crate::audit::AuditLog;
use crate::auth::require_can_transact;
use uuid::Uuid;
use chrono::Utc;

pub struct StocktakeCountParams {
    pub item_id: Uuid,
    pub location_id: Uuid,
    pub condition: ConditionCode,
    pub counted_qty: i64,
    pub notes: String,
}

pub fn record_stocktake_count(db: &Database, user: &User, params: StocktakeCountParams) -> Result<Transaction> {
    require_can_transact(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;
    let expected = db.get_balance(params.item_id, params.location_id, &params.condition)?;
    let variance = params.counted_qty - expected;

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::StocktakeCount,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: Some(params.location_id),
        to_location_id: Some(params.location_id),
        quantity: params.counted_qty,
        serial_ids: vec![],
        lot_id: None,
        user_id: user.id,
        user_name: user.display_name.clone(),
        approved_by_id: None,
        approved_by_name: None,
        reference: String::new(),
        reason: format!("Expected: {}, Counted: {}, Variance: {}", expected, params.counted_qty, variance),
        notes: params.notes.clone(),
        requires_approval: variance != 0,
        approved: variance == 0,
        timestamp: Utc::now(),
    };
    db.save_transaction(&tx)?;

    let prev_hash = db.get_last_audit_hash()?;
    let entry = AuditLog::create_entry(
        user.id, &user.display_name,
        "STOCKTAKE_COUNT", "Transaction", tx.id,
        &format!("Stocktake: {} expected={} counted={} variance={}", item.description, expected, params.counted_qty, variance),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}

pub fn apply_stocktake_adjustment(db: &Database, user: &User, params: StocktakeCountParams) -> Result<Transaction> {
    require_can_transact(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;
    let expected = db.get_balance(params.item_id, params.location_id, &params.condition)?;
    let variance = params.counted_qty - expected;

    if variance != 0 {
        db.upsert_balance(params.item_id, params.location_id, &params.condition, variance)?;
    }

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::StocktakeAdjust,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: Some(params.location_id),
        to_location_id: Some(params.location_id),
        quantity: variance,
        serial_ids: vec![],
        lot_id: None,
        user_id: user.id,
        user_name: user.display_name.clone(),
        approved_by_id: Some(user.id),
        approved_by_name: Some(user.display_name.clone()),
        reference: String::new(),
        reason: format!("Stocktake adjustment: variance {}", variance),
        notes: params.notes.clone(),
        requires_approval: true,
        approved: true,
        timestamp: Utc::now(),
    };
    db.save_transaction(&tx)?;

    let prev_hash = db.get_last_audit_hash()?;
    let entry = AuditLog::create_entry(
        user.id, &user.display_name,
        "STOCKTAKE_ADJUST", "Transaction", tx.id,
        &format!("Stocktake adjust: {} variance={}", item.description, variance),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}
