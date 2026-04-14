use crate::db::Database;
use crate::models::*;
use crate::error::Result;
use crate::audit::AuditLog;
use crate::auth::require_can_transact;
use uuid::Uuid;
use chrono::Utc;

pub struct IssueParams {
    pub item_id: Uuid,
    pub from_location_id: Uuid,
    pub quantity: i64,
    pub condition: ConditionCode,
    pub custodian_name: String,
    pub rank: String,
    pub unit: String,
    pub reference: String,
    pub notes: String,
}

pub fn issue(db: &Database, user: &User, params: IssueParams) -> Result<Transaction> {
    require_can_transact(user)?;
    let item = db.get_item(params.item_id)?.ok_or_else(|| crate::error::StoremanError::NotFound("Item not found".into()))?;

    db.upsert_balance(params.item_id, params.from_location_id, &params.condition, -params.quantity)?;

    let tx = Transaction {
        id: Uuid::new_v4(),
        transaction_type: TransactionType::Issue,
        item_id: params.item_id,
        item_description: item.description.clone(),
        from_location_id: Some(params.from_location_id),
        to_location_id: None,
        quantity: params.quantity,
        serial_ids: vec![],
        lot_id: None,
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

    let custody = CustodyRecord {
        id: Uuid::new_v4(),
        item_id: params.item_id,
        serial_id: None,
        lot_id: None,
        custodian_id: user.id,
        custodian_name: params.custodian_name.clone(),
        rank: params.rank.clone(),
        unit: params.unit.clone(),
        quantity: params.quantity,
        issued_at: Utc::now(),
        returned_at: None,
        transaction_id: tx.id,
        status: CustodyStatus::Active,
        notes: params.notes.clone(),
    };
    db.create_custody(&custody)?;

    let prev_hash = db.get_last_audit_hash()?;
    let entry = AuditLog::create_entry(
        user.id, &user.display_name,
        "ISSUE", "Transaction", tx.id,
        &format!("Issued {} x {} to {} {}", params.quantity, item.description, params.rank, params.custodian_name),
        &prev_hash,
    );
    db.save_audit_entry(&entry)?;

    Ok(tx)
}
