use rusqlite::Connection;
use crate::error::Result;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            role TEXT NOT NULL,
            rank TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            last_login TEXT
        );

        CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY,
            barcode TEXT,
            nsn TEXT,
            part_number TEXT,
            description TEXT NOT NULL,
            category TEXT NOT NULL DEFAULT '',
            item_type TEXT NOT NULL,
            unit_of_issue TEXT NOT NULL DEFAULT 'EA',
            controlled_category TEXT NOT NULL DEFAULT 'None',
            reorder_point INTEGER,
            shelf_life_days INTEGER,
            notes TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS locations (
            id TEXT PRIMARY KEY,
            warehouse TEXT NOT NULL,
            aisle TEXT NOT NULL DEFAULT '',
            rack TEXT NOT NULL DEFAULT '',
            bin TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            capacity_note TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS stock_balances (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL REFERENCES items(id),
            location_id TEXT NOT NULL REFERENCES locations(id),
            condition TEXT NOT NULL DEFAULT 'Serviceable',
            quantity INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL,
            UNIQUE(item_id, location_id, condition)
        );

        CREATE TABLE IF NOT EXISTS serial_records (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL REFERENCES items(id),
            serial_number TEXT NOT NULL,
            condition TEXT NOT NULL DEFAULT 'Serviceable',
            location_id TEXT REFERENCES locations(id),
            custodian_id TEXT REFERENCES users(id),
            inspection_due TEXT,
            notes TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS lot_records (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL REFERENCES items(id),
            lot_number TEXT NOT NULL,
            expiry_date TEXT,
            quantity INTEGER NOT NULL DEFAULT 0,
            location_id TEXT REFERENCES locations(id),
            received_at TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS custody_records (
            id TEXT PRIMARY KEY,
            item_id TEXT NOT NULL REFERENCES items(id),
            serial_id TEXT REFERENCES serial_records(id),
            lot_id TEXT REFERENCES lot_records(id),
            custodian_id TEXT NOT NULL REFERENCES users(id),
            custodian_name TEXT NOT NULL,
            rank TEXT NOT NULL DEFAULT '',
            unit TEXT NOT NULL DEFAULT '',
            quantity INTEGER NOT NULL DEFAULT 1,
            issued_at TEXT NOT NULL,
            returned_at TEXT,
            transaction_id TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Active',
            notes TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            transaction_type TEXT NOT NULL,
            item_id TEXT NOT NULL REFERENCES items(id),
            item_description TEXT NOT NULL,
            from_location_id TEXT REFERENCES locations(id),
            to_location_id TEXT REFERENCES locations(id),
            quantity INTEGER NOT NULL,
            serial_ids TEXT NOT NULL DEFAULT '[]',
            lot_id TEXT REFERENCES lot_records(id),
            user_id TEXT NOT NULL REFERENCES users(id),
            user_name TEXT NOT NULL,
            approved_by_id TEXT REFERENCES users(id),
            approved_by_name TEXT,
            reference TEXT NOT NULL DEFAULT '',
            reason TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            requires_approval INTEGER NOT NULL DEFAULT 0,
            approved INTEGER NOT NULL DEFAULT 1,
            timestamp TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id TEXT NOT NULL UNIQUE,
            timestamp TEXT NOT NULL,
            user_id TEXT NOT NULL,
            user_name TEXT NOT NULL,
            action TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            details TEXT NOT NULL DEFAULT '',
            previous_hash TEXT NOT NULL DEFAULT '',
            entry_hash TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS controlled_profiles (
            id TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            requires_dual_approval INTEGER NOT NULL DEFAULT 0,
            restricted_export INTEGER NOT NULL DEFAULT 0,
            watermark_text TEXT NOT NULL DEFAULT '',
            mandatory_custody_fields INTEGER NOT NULL DEFAULT 0,
            audit_all_views INTEGER NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS stocktake_sessions (
            id TEXT PRIMARY KEY,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            started_by TEXT NOT NULL REFERENCES users(id),
            notes TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'Open'
        );

        CREATE TABLE IF NOT EXISTS stocktake_counts (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES stocktake_sessions(id),
            item_id TEXT NOT NULL REFERENCES items(id),
            location_id TEXT NOT NULL REFERENCES locations(id),
            condition TEXT NOT NULL DEFAULT 'Serviceable',
            expected_qty INTEGER NOT NULL DEFAULT 0,
            counted_qty INTEGER NOT NULL DEFAULT 0,
            variance INTEGER NOT NULL DEFAULT 0,
            counted_by TEXT NOT NULL REFERENCES users(id),
            counted_at TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT ''
        );
    ")?;

    seed_defaults(conn)?;
    Ok(())
}

fn seed_defaults(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;
    use rand::rngs::OsRng;

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(b"admin", &salt)
        .map(|h| h.to_string())
        .unwrap_or_default();

    let admin_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO users (id, username, display_name, role, rank, active, password_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![admin_id, "admin", "System Admin", "Admin", "N/A", 1, hash, now],
    )?;

    // Default locations
    let loc_id1 = uuid::Uuid::new_v4().to_string();
    let loc_id2 = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO locations (id, warehouse, aisle, rack, bin, description, active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![loc_id1, "WH-1", "A", "01", "01", "Main Warehouse Row A", 1],
    )?;
    conn.execute(
        "INSERT INTO locations (id, warehouse, aisle, rack, bin, description, active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![loc_id2, "WH-1", "B", "01", "01", "Main Warehouse Row B", 1],
    )?;

    Ok(())
}
