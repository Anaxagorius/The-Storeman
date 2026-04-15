use crate::db::Database;
use crate::models::*;
use crate::error::Result;
use crate::audit::AuditLog;
use crate::auth::require_can_transact;
use uuid::Uuid;
use chrono::Utc;

pub struct ReceiveParams {
    pub item_id: Uuid,
    pub to_location_id: Uuid,
    pub quantity: i64,
    pub condition: ConditionCode,
    pub lot_number: Option<String>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub serial_numbers: Vec<String>,
    pub reference: String,
    pub notes: String,
}

pub fn receive(db: &Database, user: &User, params: ReceiveParams) -> Result<Transaction> {
    require_can_transact(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;

    let lot_id = if let Some(lot_num) = &params.lot_number {
        let lot = LotRecord {
            id: Uuid::new_v4(),
            item_id: params.item_id,
            lot_number: lot_num.clone(),
            expiry_date: params.expiry_date,
            quantity: params.quantity,
            location_id: Some(params.to_location_id),
            received_at: Utc::now(),
            notes: params.notes.clone(),
            active: true,
        };
        db.create_lot(&lot)?;
        Some(lot.id)
    } else {
        None
    };

    let mut serial_ids = Vec::new();
    for sn in &params.serial_numbers {
        let serial = SerialRecord {
            id: Uuid::new_v4(),
            item_id: params.item_id,
            serial_number: sn.clone(),
            condition: params.condition.clone(),
            location_id: Some(params.to_location_id),
            custodian_id: None,
            inspection_due: None,
            notes: String::new(),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.create_serial(&serial)?;
        serial_ids.push(serial.id);
    }

    db.upsert_balance(params.item_id, params.to_location_id, &params.condition, params.quantity)?;

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::Receive,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: None,
        to_location_id: Some(params.to_location_id),
        quantity: params.quantity,
        serial_ids: serial_ids.clone(),
        lot_id,
        user_id: user.id,
        user_name: user.display_name.clone(),
        approved_by_id: None,
        approved_by_name: None,
        reference: params.reference.clone(),
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
        "RECEIVE", "Transaction", tx.id,
        &format!("Received {} x {} to {}", params.quantity, item.description, params.to_location_id),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}
