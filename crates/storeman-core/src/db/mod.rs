pub mod migrations;

use rusqlite::{Connection, params};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::{Result, StoremanError};
use crate::models::*;
use crate::audit::AuditEntry;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        migrations::run_migrations(&conn)?;
        Ok(Database { conn })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        migrations::run_migrations(&conn)?;
        Ok(Database { conn })
    }

    // ── Users ──────────────────────────────────────────────────────────────────

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<(User, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, username, display_name, role, rank, active, password_hash, created_at, last_login
             FROM users WHERE username = ?1"
        )?;
        let result = stmt.query_row(params![username], |row| {
            let role_str: String = row.get(3)?;
            let active: i64 = row.get(5)?;
            let hash: String = row.get(6)?;
            let created_at: String = row.get(7)?;
            let last_login: Option<String> = row.get(8)?;
            Ok((
                User {
                    id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    role: parse_role(&role_str),
                    rank: row.get(4)?,
                    active: active != 0,
                    created_at: parse_dt(&created_at),
                    last_login: last_login.as_deref().map(parse_dt),
                },
                hash,
            ))
        });
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, username, display_name, role, rank, active, created_at, last_login
             FROM users WHERE id = ?1"
        )?;
        let result = stmt.query_row(params![id.to_string()], |row| {
            let role_str: String = row.get(3)?;
            let active: i64 = row.get(5)?;
            let created_at: String = row.get(6)?;
            let last_login: Option<String> = row.get(7)?;
            Ok(User {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                username: row.get(1)?,
                display_name: row.get(2)?,
                role: parse_role(&role_str),
                rank: row.get(4)?,
                active: active != 0,
                created_at: parse_dt(&created_at),
                last_login: last_login.as_deref().map(parse_dt),
            })
        });
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, username, display_name, role, rank, active, created_at, last_login FROM users ORDER BY display_name"
        )?;
        let rows = stmt.query_map([], |row| {
            let role_str: String = row.get(3)?;
            let active: i64 = row.get(5)?;
            let created_at: String = row.get(6)?;
            let last_login: Option<String> = row.get(7)?;
            Ok(User {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                username: row.get(1)?,
                display_name: row.get(2)?,
                role: parse_role(&role_str),
                rank: row.get(4)?,
                active: active != 0,
                created_at: parse_dt(&created_at),
                last_login: last_login.as_deref().map(parse_dt),
            })
        })?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn create_user(&self, user: &User, password_hash: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO users (id, username, display_name, role, rank, active, password_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                user.id.to_string(), user.username, user.display_name,
                role_to_str(&user.role), user.rank,
                user.active as i64, password_hash,
                user.created_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn update_user(&self, user: &User) -> Result<()> {
        self.conn.execute(
            "UPDATE users SET display_name=?2, role=?3, rank=?4, active=?5 WHERE id=?1",
            params![
                user.id.to_string(), user.display_name,
                role_to_str(&user.role), user.rank, user.active as i64
            ],
        )?;
        Ok(())
    }

    pub fn update_user_password(&self, user_id: Uuid, hash: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE users SET password_hash=?2 WHERE id=?1",
            params![user_id.to_string(), hash],
        )?;
        Ok(())
    }

    pub fn update_last_login(&self, user_id: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE users SET last_login=?2 WHERE id=?1",
            params![user_id.to_string(), now],
        )?;
        Ok(())
    }

    // ── Items ──────────────────────────────────────────────────────────────────

    pub fn get_item(&self, id: Uuid) -> Result<Option<Item>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, barcode, nsn, part_number, description, category, item_type, unit_of_issue,
             controlled_category, reorder_point, shelf_life_days, notes, active, created_at, updated_at,
             equipment_variant_id
             FROM items WHERE id = ?1"
        )?;
        let result = stmt.query_row(params![id.to_string()], row_to_item);
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_items(&self, active_only: bool) -> Result<Vec<Item>> {
        let sql = if active_only {
            "SELECT id, barcode, nsn, part_number, description, category, item_type, unit_of_issue,
             controlled_category, reorder_point, shelf_life_days, notes, active, created_at, updated_at,
             equipment_variant_id
             FROM items WHERE active=1 ORDER BY description"
        } else {
            "SELECT id, barcode, nsn, part_number, description, category, item_type, unit_of_issue,
             controlled_category, reorder_point, shelf_life_days, notes, active, created_at, updated_at,
             equipment_variant_id
             FROM items ORDER BY description"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], row_to_item)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn create_item(&self, item: &Item) -> Result<()> {
        self.conn.execute(
            "INSERT INTO items (id, barcode, nsn, part_number, description, category, item_type,
             unit_of_issue, controlled_category, reorder_point, shelf_life_days, notes, active,
             created_at, updated_at, equipment_variant_id)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                item.id.to_string(), item.barcode, item.nsn, item.part_number,
                item.description, item.category,
                item.item_type.to_string(), item.unit_of_issue,
                item.controlled_category.to_string(),
                item.reorder_point, item.shelf_life_days, item.notes,
                item.active as i64,
                item.created_at.to_rfc3339(), item.updated_at.to_rfc3339(),
                item.equipment_variant_id.map(|u| u.to_string()),
            ],
        )?;
        Ok(())
    }

    pub fn update_item(&self, item: &Item) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE items SET barcode=?2, nsn=?3, part_number=?4, description=?5, category=?6,
             item_type=?7, unit_of_issue=?8, controlled_category=?9, reorder_point=?10,
             shelf_life_days=?11, notes=?12, active=?13, updated_at=?14,
             equipment_variant_id=?15 WHERE id=?1",
            params![
                item.id.to_string(), item.barcode, item.nsn, item.part_number,
                item.description, item.category,
                item.item_type.to_string(), item.unit_of_issue,
                item.controlled_category.to_string(),
                item.reorder_point, item.shelf_life_days, item.notes,
                item.active as i64, now,
                item.equipment_variant_id.map(|u| u.to_string()),
            ],
        )?;
        Ok(())
    }

    // ── Locations ──────────────────────────────────────────────────────────────

    pub fn list_locations(&self) -> Result<Vec<Location>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, warehouse, aisle, rack, bin, description, capacity_note, active FROM locations ORDER BY warehouse, aisle, rack, bin"
        )?;
        let rows = stmt.query_map([], row_to_location)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_location(&self, id: Uuid) -> Result<Option<Location>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, warehouse, aisle, rack, bin, description, capacity_note, active FROM locations WHERE id=?1"
        )?;
        let result = stmt.query_row(params![id.to_string()], row_to_location);
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_location(&self, loc: &Location) -> Result<()> {
        self.conn.execute(
            "INSERT INTO locations (id, warehouse, aisle, rack, bin, description, capacity_note, active)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![loc.id.to_string(), loc.warehouse, loc.aisle, loc.rack, loc.bin,
                    loc.description, loc.capacity_note, loc.active as i64],
        )?;
        Ok(())
    }

    pub fn update_location(&self, loc: &Location) -> Result<()> {
        self.conn.execute(
            "UPDATE locations SET warehouse=?2, aisle=?3, rack=?4, bin=?5, description=?6, capacity_note=?7, active=?8 WHERE id=?1",
            params![loc.id.to_string(), loc.warehouse, loc.aisle, loc.rack, loc.bin,
                    loc.description, loc.capacity_note, loc.active as i64],
        )?;
        Ok(())
    }

    // ── Stock Balances ─────────────────────────────────────────────────────────

    pub fn get_balance(&self, item_id: Uuid, location_id: Uuid, condition: &ConditionCode) -> Result<i64> {
        let result = self.conn.query_row(
            "SELECT quantity FROM stock_balances WHERE item_id=?1 AND location_id=?2 AND condition=?3",
            params![item_id.to_string(), location_id.to_string(), condition.to_string()],
            |r| r.get::<_, i64>(0),
        );
        match result {
            Ok(qty) => Ok(qty),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    pub fn upsert_balance(&self, item_id: Uuid, location_id: Uuid, condition: &ConditionCode, delta: i64) -> Result<()> {
        let existing = self.get_balance(item_id, location_id, condition)?;
        let new_qty = existing + delta;
        if new_qty < 0 {
            return Err(StoremanError::InsufficientStock { available: existing, requested: -delta });
        }
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO stock_balances (id, item_id, location_id, condition, quantity, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(item_id, location_id, condition) DO UPDATE SET quantity=?5, updated_at=?6",
            params![
                Uuid::new_v4().to_string(),
                item_id.to_string(), location_id.to_string(),
                condition.to_string(), new_qty, now
            ],
        )?;
        Ok(())
    }

    pub fn list_balances_for_item(&self, item_id: Uuid) -> Result<Vec<StockBalance>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, location_id, condition, quantity, updated_at FROM stock_balances WHERE item_id=?1"
        )?;
        let rows = stmt.query_map(params![item_id.to_string()], row_to_balance)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn list_all_balances(&self) -> Result<Vec<StockBalance>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, location_id, condition, quantity, updated_at FROM stock_balances ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], row_to_balance)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    // ── Serials ────────────────────────────────────────────────────────────────

    pub fn create_serial(&self, s: &SerialRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO serial_records (id, item_id, serial_number, condition, location_id, custodian_id, inspection_due, notes, active, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![
                s.id.to_string(), s.item_id.to_string(), s.serial_number,
                s.condition.to_string(),
                s.location_id.map(|u| u.to_string()),
                s.custodian_id.map(|u| u.to_string()),
                s.inspection_due.map(|d| d.to_rfc3339()),
                s.notes, s.active as i64,
                s.created_at.to_rfc3339(), s.updated_at.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn list_serials_for_item(&self, item_id: Uuid) -> Result<Vec<SerialRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, serial_number, condition, location_id, custodian_id, inspection_due, notes, active, created_at, updated_at
             FROM serial_records WHERE item_id=?1 AND active=1"
        )?;
        let rows = stmt.query_map(params![item_id.to_string()], row_to_serial)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    // ── Lots ───────────────────────────────────────────────────────────────────

    pub fn create_lot(&self, lot: &LotRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO lot_records (id, item_id, lot_number, expiry_date, quantity, location_id, received_at, notes, active)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                lot.id.to_string(), lot.item_id.to_string(), lot.lot_number,
                lot.expiry_date.map(|d| d.to_string()),
                lot.quantity,
                lot.location_id.map(|u| u.to_string()),
                lot.received_at.to_rfc3339(), lot.notes, lot.active as i64
            ],
        )?;
        Ok(())
    }

    pub fn list_lots_for_item(&self, item_id: Uuid) -> Result<Vec<LotRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, lot_number, expiry_date, quantity, location_id, received_at, notes, active
             FROM lot_records WHERE item_id=?1 AND active=1"
        )?;
        let rows = stmt.query_map(params![item_id.to_string()], row_to_lot)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn list_expiring_lots(&self, within_days: i64) -> Result<Vec<LotRecord>> {
        let cutoff = (Utc::now() + chrono::Duration::days(within_days)).date_naive().to_string();
        let today = Utc::now().date_naive().to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, lot_number, expiry_date, quantity, location_id, received_at, notes, active
             FROM lot_records WHERE active=1 AND expiry_date IS NOT NULL AND expiry_date >= ?1 AND expiry_date <= ?2"
        )?;
        let rows = stmt.query_map(params![today, cutoff], row_to_lot)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    // ── Custody ────────────────────────────────────────────────────────────────

    pub fn create_custody(&self, c: &CustodyRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO custody_records (id, item_id, serial_id, lot_id, custodian_id, custodian_name, rank, unit, quantity, issued_at, returned_at, transaction_id, status, notes)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                c.id.to_string(), c.item_id.to_string(),
                c.serial_id.map(|u| u.to_string()),
                c.lot_id.map(|u| u.to_string()),
                c.custodian_id.to_string(), c.custodian_name, c.rank, c.unit,
                c.quantity, c.issued_at.to_rfc3339(),
                c.returned_at.map(|d| d.to_rfc3339()),
                c.transaction_id.to_string(),
                c.status.to_string(), c.notes
            ],
        )?;
        Ok(())
    }

    pub fn close_custody(&self, custody_id: Uuid) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE custody_records SET status='Returned', returned_at=?2 WHERE id=?1",
            params![custody_id.to_string(), now],
        )?;
        Ok(())
    }

    pub fn list_active_custody(&self) -> Result<Vec<CustodyRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, serial_id, lot_id, custodian_id, custodian_name, rank, unit, quantity,
             issued_at, returned_at, transaction_id, status, notes
             FROM custody_records WHERE status='Active' ORDER BY issued_at DESC"
        )?;
        let rows = stmt.query_map([], row_to_custody)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn list_all_custody(&self) -> Result<Vec<CustodyRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, item_id, serial_id, lot_id, custodian_id, custodian_name, rank, unit, quantity,
             issued_at, returned_at, transaction_id, status, notes
             FROM custody_records ORDER BY issued_at DESC"
        )?;
        let rows = stmt.query_map([], row_to_custody)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    // ── Transactions ───────────────────────────────────────────────────────────

    pub fn save_transaction(&self, tx: &Transaction) -> Result<()> {
        let serial_ids_json = serde_json::to_string(&tx.serial_ids).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO transactions (id, transaction_type, item_id, item_description,
             from_location_id, to_location_id, quantity, serial_ids, lot_id,
             user_id, user_name, approved_by_id, approved_by_name,
             reference, reason, notes, requires_approval, approved, timestamp)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
            params![
                tx.id.to_string(), tx.transaction_type.to_string(),
                tx.item_id.to_string(), tx.item_description,
                tx.from_location_id.map(|u| u.to_string()),
                tx.to_location_id.map(|u| u.to_string()),
                tx.quantity, serial_ids_json,
                tx.lot_id.map(|u| u.to_string()),
                tx.user_id.to_string(), tx.user_name,
                tx.approved_by_id.map(|u| u.to_string()),
                tx.approved_by_name.as_deref(),
                tx.reference, tx.reason, tx.notes,
                tx.requires_approval as i64, tx.approved as i64,
                tx.timestamp.to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn list_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, transaction_type, item_id, item_description,
             from_location_id, to_location_id, quantity, serial_ids, lot_id,
             user_id, user_name, approved_by_id, approved_by_name,
             reference, reason, notes, requires_approval, approved, timestamp
             FROM transactions ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], row_to_transaction)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn list_transactions_for_item(&self, item_id: Uuid) -> Result<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, transaction_type, item_id, item_description,
             from_location_id, to_location_id, quantity, serial_ids, lot_id,
             user_id, user_name, approved_by_id, approved_by_name,
             reference, reason, notes, requires_approval, approved, timestamp
             FROM transactions WHERE item_id=?1 ORDER BY timestamp DESC"
        )?;
        let rows = stmt.query_map(params![item_id.to_string()], row_to_transaction)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    // ── Audit ──────────────────────────────────────────────────────────────────

    pub fn save_audit_entry(&self, entry: &AuditEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO audit_log (entry_id, timestamp, user_id, user_name, action, entity_type, entity_id, details, previous_hash, entry_hash)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                entry.entry_id.to_string(), entry.timestamp.to_rfc3339(),
                entry.user_id.to_string(), entry.user_name,
                entry.action, entry.entity_type,
                entry.entity_id.to_string(), entry.details,
                entry.previous_hash, entry.entry_hash
            ],
        )?;
        Ok(())
    }

    pub fn get_last_audit_hash(&self) -> Result<String> {
        let result = self.conn.query_row(
            "SELECT entry_hash FROM audit_log ORDER BY id DESC LIMIT 1",
            [], |r| r.get::<_, String>(0),
        );
        match result {
            Ok(h) => Ok(h),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok("GENESIS".to_string()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_audit_entries(&self, limit: usize) -> Result<Vec<AuditEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT entry_id, timestamp, user_id, user_name, action, entity_type, entity_id, details, previous_hash, entry_hash
             FROM audit_log ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(AuditEntry {
                entry_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                timestamp: parse_dt(&row.get::<_, String>(1)?),
                user_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                user_name: row.get(3)?,
                action: row.get(4)?,
                entity_type: row.get(5)?,
                entity_id: Uuid::parse_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                details: row.get(7)?,
                previous_hash: row.get(8)?,
                entry_hash: row.get(9)?,
            })
        })?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    // ── Reports ────────────────────────────────────────────────────────────────

    pub fn export_stock_csv(&self) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(["Item ID", "Description", "Location", "Condition", "Quantity", "Updated At"])
            .map_err(|e| StoremanError::Other(e.to_string()))?;

        let mut stmt = self.conn.prepare(
            "SELECT sb.item_id, i.description, l.warehouse||'-'||l.aisle||'-'||l.rack||'-'||l.bin,
             sb.condition, sb.quantity, sb.updated_at
             FROM stock_balances sb
             JOIN items i ON i.id = sb.item_id
             JOIN locations l ON l.id = sb.location_id
             ORDER BY i.description"
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;

        for row in rows {
            let (id, desc, loc, cond, qty, updated) = row?;
            wtr.write_record([&id, &desc, &loc, &cond, &qty.to_string(), &updated])
                .map_err(|e| StoremanError::Other(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| StoremanError::Other(e.to_string()))?;
        String::from_utf8(data).map_err(|e| StoremanError::Other(e.to_string()))
    }

    pub fn export_transactions_csv(&self, limit: usize) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(["ID", "Type", "Item", "Qty", "User", "Reference", "Timestamp"])
            .map_err(|e| StoremanError::Other(e.to_string()))?;

        let txs = self.list_transactions(limit)?;
        for tx in &txs {
            wtr.write_record([
                &tx.id.to_string(),
                &tx.transaction_type.to_string(),
                &tx.item_description,
                &tx.quantity.to_string(),
                &tx.user_name,
                &tx.reference,
                &tx.timestamp.to_rfc3339(),
            ]).map_err(|e| StoremanError::Other(e.to_string()))?;
        }

        let data = wtr.into_inner().map_err(|e| StoremanError::Other(e.to_string()))?;
        String::from_utf8(data).map_err(|e| StoremanError::Other(e.to_string()))
    }

    pub fn items_below_reorder(&self) -> Result<Vec<(Item, i64)>> {
        let items = self.list_items(true)?;
        let mut result = Vec::new();
        for item in items {
            if let Some(rp) = item.reorder_point {
                let total: i64 = self.conn.query_row(
                    "SELECT COALESCE(SUM(quantity), 0) FROM stock_balances WHERE item_id=?1 AND condition='Serviceable'",
                    params![item.id.to_string()],
                    |r| r.get(0),
                )?;
                if total <= rp {
                    result.push((item, total));
                }
            }
        }
        Ok(result)
    }

    // ── Master Equipment Reference ─────────────────────────────────────────────

    pub fn list_equipment_items(&self, active_only: bool) -> Result<Vec<EquipmentItem>> {
        let sql = if active_only {
            "SELECT id, common_name, official_designation, equipment_category,
              nato_category_code, manufacturer, country_of_origin, service_branch,
              status, introduction_year, notes, active
             FROM equipment_items WHERE active=1 ORDER BY equipment_category, common_name"
        } else {
            "SELECT id, common_name, official_designation, equipment_category,
              nato_category_code, manufacturer, country_of_origin, service_branch,
              status, introduction_year, notes, active
             FROM equipment_items ORDER BY equipment_category, common_name"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], row_to_equipment_item)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_equipment_item(&self, id: Uuid) -> Result<Option<EquipmentItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, common_name, official_designation, equipment_category,
              nato_category_code, manufacturer, country_of_origin, service_branch,
              status, introduction_year, notes, active
             FROM equipment_items WHERE id=?1",
        )?;
        let result = stmt.query_row(params![id.to_string()], row_to_equipment_item);
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_equipment_item(&self, eq: &EquipmentItem) -> Result<()> {
        self.conn.execute(
            "INSERT INTO equipment_items
             (id, common_name, official_designation, equipment_category,
              nato_category_code, manufacturer, country_of_origin, service_branch,
              status, introduction_year, notes, active)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![
                eq.id.to_string(),
                eq.common_name,
                eq.official_designation,
                eq.equipment_category.to_string(),
                eq.nato_category_code,
                eq.manufacturer,
                eq.country_of_origin,
                eq.service_branch.to_string(),
                eq.status.to_string(),
                eq.introduction_year,
                eq.notes,
                eq.active as i64,
            ],
        )?;
        Ok(())
    }

    pub fn update_equipment_item(&self, eq: &EquipmentItem) -> Result<()> {
        self.conn.execute(
            "UPDATE equipment_items SET
              common_name=?2, official_designation=?3, equipment_category=?4,
              nato_category_code=?5, manufacturer=?6, country_of_origin=?7,
              service_branch=?8, status=?9, introduction_year=?10, notes=?11, active=?12
             WHERE id=?1",
            params![
                eq.id.to_string(),
                eq.common_name,
                eq.official_designation,
                eq.equipment_category.to_string(),
                eq.nato_category_code,
                eq.manufacturer,
                eq.country_of_origin,
                eq.service_branch.to_string(),
                eq.status.to_string(),
                eq.introduction_year,
                eq.notes,
                eq.active as i64,
            ],
        )?;
        Ok(())
    }

    pub fn list_equipment_variants(&self, equipment_id: Uuid) -> Result<Vec<EquipmentVariant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, equipment_id, variant_name, calibre_or_spec, compatible_accessories, notes
             FROM equipment_variants WHERE equipment_id=?1 ORDER BY variant_name",
        )?;
        let rows = stmt.query_map(params![equipment_id.to_string()], row_to_equipment_variant)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn list_all_equipment_variants(&self) -> Result<Vec<EquipmentVariant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, equipment_id, variant_name, calibre_or_spec, compatible_accessories, notes
             FROM equipment_variants ORDER BY variant_name",
        )?;
        let rows = stmt.query_map([], row_to_equipment_variant)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn get_equipment_variant(&self, id: Uuid) -> Result<Option<EquipmentVariant>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, equipment_id, variant_name, calibre_or_spec, compatible_accessories, notes
             FROM equipment_variants WHERE id=?1",
        )?;
        let result = stmt.query_row(params![id.to_string()], row_to_equipment_variant);
        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_equipment_variant(&self, v: &EquipmentVariant) -> Result<()> {
        self.conn.execute(
            "INSERT INTO equipment_variants
             (id, equipment_id, variant_name, calibre_or_spec, compatible_accessories, notes)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                v.id.to_string(),
                v.equipment_id.to_string(),
                v.variant_name,
                v.calibre_or_spec,
                v.compatible_accessories,
                v.notes,
            ],
        )?;
        Ok(())
    }

    pub fn list_nato_references(&self, variant_id: Uuid) -> Result<Vec<NatoReference>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, variant_id, nsn, nato_reporting_name
             FROM nato_references WHERE variant_id=?1",
        )?;
        let rows = stmt.query_map(params![variant_id.to_string()], row_to_nato_reference)?;
        rows.map(|r| r.map_err(Into::into)).collect()
    }

    pub fn upsert_nato_reference(&self, nr: &NatoReference) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO nato_references (id, variant_id, nsn, nato_reporting_name)
             VALUES (?1,?2,?3,?4)",
            params![
                nr.id.to_string(),
                nr.variant_id.to_string(),
                nr.nsn,
                nr.nato_reporting_name,
            ],
        )?;
        Ok(())
    }
}

// ── Row mappers ────────────────────────────────────────────────────────────────

fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    use crate::models::item::{parse_controlled_category, parse_item_type};
    let active: i64 = row.get(12)?;
    Ok(Item {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        barcode: row.get(1)?,
        nsn: row.get(2)?,
        part_number: row.get(3)?,
        description: row.get(4)?,
        category: row.get(5)?,
        item_type: parse_item_type(&row.get::<_, String>(6)?),
        unit_of_issue: row.get(7)?,
        controlled_category: parse_controlled_category(&row.get::<_, String>(8)?),
        reorder_point: row.get(9)?,
        shelf_life_days: row.get(10)?,
        notes: row.get(11)?,
        active: active != 0,
        created_at: parse_dt(&row.get::<_, String>(13)?),
        updated_at: parse_dt(&row.get::<_, String>(14)?),
        equipment_variant_id: row.get::<_, Option<String>>(15)?
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok()),
    })
}

fn row_to_location(row: &rusqlite::Row<'_>) -> rusqlite::Result<Location> {
    let active: i64 = row.get(7)?;
    Ok(Location {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        warehouse: row.get(1)?,
        aisle: row.get(2)?,
        rack: row.get(3)?,
        bin: row.get(4)?,
        description: row.get(5)?,
        capacity_note: row.get(6)?,
        active: active != 0,
    })
}

fn row_to_balance(row: &rusqlite::Row<'_>) -> rusqlite::Result<StockBalance> {
    use crate::models::balance::parse_condition_code;
    Ok(StockBalance {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        item_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
        location_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap_or_default(),
        condition: parse_condition_code(&row.get::<_, String>(3)?),
        quantity: row.get(4)?,
        updated_at: parse_dt(&row.get::<_, String>(5)?),
    })
}

fn row_to_serial(row: &rusqlite::Row<'_>) -> rusqlite::Result<SerialRecord> {
    use crate::models::balance::parse_condition_code;
    let active: i64 = row.get(8)?;
    Ok(SerialRecord {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        item_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
        serial_number: row.get(2)?,
        condition: parse_condition_code(&row.get::<_, String>(3)?),
        location_id: row.get::<_, Option<String>>(4)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        custodian_id: row.get::<_, Option<String>>(5)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        inspection_due: row.get::<_, Option<String>>(6)?.as_deref().map(parse_dt),
        notes: row.get(7)?,
        active: active != 0,
        created_at: parse_dt(&row.get::<_, String>(9)?),
        updated_at: parse_dt(&row.get::<_, String>(10)?),
    })
}

fn row_to_lot(row: &rusqlite::Row<'_>) -> rusqlite::Result<LotRecord> {
    use chrono::NaiveDate;
    let active: i64 = row.get(8)?;
    Ok(LotRecord {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        item_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
        lot_number: row.get(2)?,
        expiry_date: row.get::<_, Option<String>>(3)?.as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()),
        quantity: row.get(4)?,
        location_id: row.get::<_, Option<String>>(5)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        received_at: parse_dt(&row.get::<_, String>(6)?),
        notes: row.get(7)?,
        active: active != 0,
    })
}

fn row_to_custody(row: &rusqlite::Row<'_>) -> rusqlite::Result<CustodyRecord> {
    let status_str: String = row.get(12)?;
    let status = match status_str.as_str() {
        "Active" => CustodyStatus::Active,
        "Returned" => CustodyStatus::Returned,
        "Lost" => CustodyStatus::Lost,
        _ => CustodyStatus::Disposed,
    };
    Ok(CustodyRecord {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        item_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
        serial_id: row.get::<_, Option<String>>(2)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        lot_id: row.get::<_, Option<String>>(3)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        custodian_id: Uuid::parse_str(&row.get::<_, String>(4)?).unwrap_or_default(),
        custodian_name: row.get(5)?,
        rank: row.get(6)?,
        unit: row.get(7)?,
        quantity: row.get(8)?,
        issued_at: parse_dt(&row.get::<_, String>(9)?),
        returned_at: row.get::<_, Option<String>>(10)?.as_deref().map(parse_dt),
        transaction_id: Uuid::parse_str(&row.get::<_, String>(11)?).unwrap_or_default(),
        status,
        notes: row.get(13)?,
    })
}

fn row_to_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Transaction> {
    let tx_type_str: String = row.get(1)?;
    let serial_ids_json: String = row.get(7)?;
    let serial_ids: Vec<Uuid> = serde_json::from_str(&serial_ids_json).unwrap_or_default();
    let req_approval: i64 = row.get(16)?;
    let approved: i64 = row.get(17)?;
    let tx_type = match tx_type_str.as_str() {
        "Receive" => TransactionType::Receive,
        "Issue" => TransactionType::Issue,
        "Return" => TransactionType::Return,
        "Transfer" => TransactionType::Transfer,
        "Adjust" => TransactionType::Adjust,
        "Dispose" => TransactionType::Dispose,
        "Stocktake Count" => TransactionType::StocktakeCount,
        "Stocktake Adjust" => TransactionType::StocktakeAdjust,
        _ => TransactionType::Adjust,
    };
    Ok(Transaction {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        transaction_type: tx_type,
        item_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap_or_default(),
        item_description: row.get(3)?,
        from_location_id: row.get::<_, Option<String>>(4)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        to_location_id: row.get::<_, Option<String>>(5)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        quantity: row.get(6)?,
        serial_ids,
        lot_id: row.get::<_, Option<String>>(8)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        user_id: Uuid::parse_str(&row.get::<_, String>(9)?).unwrap_or_default(),
        user_name: row.get(10)?,
        approved_by_id: row.get::<_, Option<String>>(11)?.as_deref().and_then(|s| Uuid::parse_str(s).ok()),
        approved_by_name: row.get(12)?,
        reference: row.get(13)?,
        reason: row.get(14)?,
        notes: row.get(15)?,
        requires_approval: req_approval != 0,
        approved: approved != 0,
        timestamp: parse_dt(&row.get::<_, String>(18)?),
    })
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn parse_dt(s: &str) -> DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn parse_role(s: &str) -> Role {
    match s {
        "Storeman" => Role::Storeman,
        "CQMS" => Role::CQMS,
        "Officer" => Role::Officer,
        "Inspector" => Role::Inspector,
        _ => Role::Admin,
    }
}

fn role_to_str(r: &Role) -> &'static str {
    match r {
        Role::Storeman => "Storeman",
        Role::CQMS => "CQMS",
        Role::Officer => "Officer",
        Role::Inspector => "Inspector",
        Role::Admin => "Admin",
    }
}

fn row_to_equipment_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<EquipmentItem> {
    use crate::models::equipment::{
        parse_equipment_category, parse_service_branch, parse_equipment_status,
    };
    let active: i64 = row.get(11)?;
    Ok(EquipmentItem {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        common_name: row.get(1)?,
        official_designation: row.get(2)?,
        equipment_category: parse_equipment_category(&row.get::<_, String>(3)?),
        nato_category_code: row.get(4)?,
        manufacturer: row.get(5)?,
        country_of_origin: row.get(6)?,
        service_branch: parse_service_branch(&row.get::<_, String>(7)?),
        status: parse_equipment_status(&row.get::<_, String>(8)?),
        introduction_year: row.get(9)?,
        notes: row.get(10)?,
        active: active != 0,
    })
}

fn row_to_equipment_variant(row: &rusqlite::Row<'_>) -> rusqlite::Result<EquipmentVariant> {
    Ok(EquipmentVariant {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        equipment_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
        variant_name: row.get(2)?,
        calibre_or_spec: row.get(3)?,
        compatible_accessories: row.get(4)?,
        notes: row.get(5)?,
    })
}

fn row_to_nato_reference(row: &rusqlite::Row<'_>) -> rusqlite::Result<NatoReference> {
    Ok(NatoReference {
        id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
        variant_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
        nsn: row.get(2)?,
        nato_reporting_name: row.get(3)?,
    })
}
