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

    // ── Master Equipment Reference tables ──────────────────────────────────────
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS equipment_items (
            id TEXT PRIMARY KEY,
            common_name TEXT NOT NULL,
            official_designation TEXT NOT NULL,
            equipment_category TEXT NOT NULL DEFAULT 'Other',
            nato_category_code TEXT,
            manufacturer TEXT,
            country_of_origin TEXT,
            service_branch TEXT NOT NULL DEFAULT 'Army',
            status TEXT NOT NULL DEFAULT 'In Service',
            introduction_year INTEGER,
            notes TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS equipment_variants (
            id TEXT PRIMARY KEY,
            equipment_id TEXT NOT NULL REFERENCES equipment_items(id),
            variant_name TEXT NOT NULL,
            calibre_or_spec TEXT,
            compatible_accessories TEXT,
            notes TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS nato_references (
            id TEXT PRIMARY KEY,
            variant_id TEXT NOT NULL REFERENCES equipment_variants(id),
            nsn TEXT,
            nato_reporting_name TEXT
        );
    ")?;

    // Add equipment_variant_id to items table — ignored if column already exists.
    let _ = conn.execute(
        "ALTER TABLE items ADD COLUMN equipment_variant_id TEXT REFERENCES equipment_variants(id)",
        [],
    );

    seed_defaults(conn)?;
    seed_equipment_reference(conn)?;
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

/// Seeds the Master Equipment Reference with unclassified OSINT data drawn from
/// publicly available sources (Wikipedia – List of equipment of the Canadian Armed
/// Forces; canada.ca – Defence Equipment pages).  This data covers individual kit,
/// weapons, vehicles, optics, and communications equipment in CAF service.
///
/// The function is idempotent: if equipment_items already contains rows it returns
/// immediately without modifying anything.
fn seed_equipment_reference(conn: &Connection) -> Result<()> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM equipment_items", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // Helper macro – inserts one equipment_item row and returns its UUID string.
    // Fields: common_name, official_designation, category, manufacturer,
    //         country_of_origin, service_branch, status, intro_year, notes
    struct EquipSeed {
        id: String,
        common_name: &'static str,
        official_designation: &'static str,
        category: &'static str,
        manufacturer: &'static str,
        country_of_origin: &'static str,
        service_branch: &'static str,
        status: &'static str,
        introduction_year: Option<i32>,
        notes: &'static str,
    }

    let items: Vec<EquipSeed> = vec![
        // ── Weapons ──────────────────────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C7A2 Rifle",
            official_designation: "Rifle, 5.56mm, C7A2",
            category: "Weapon",
            manufacturer: "Colt Canada",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2004),
            notes: "Primary service rifle of the CAF. Features flat-top upper receiver, full-length rail, and collapsible stock.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C8A3 Carbine",
            official_designation: "Carbine, 5.56mm, C8A3",
            category: "Weapon",
            manufacturer: "Colt Canada",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2019),
            notes: "Compact carbine variant; primary weapon for vehicle crews, special operations, and certain support trades.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C9A2 Light Machine Gun",
            official_designation: "Machine Gun, Light, 5.56mm, C9A2",
            category: "Weapon",
            manufacturer: "Colt Canada",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2006),
            notes: "Section-level light support weapon. Fires standard 5.56×45mm NATO ammunition from a 200-round drum or linked belt.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C6A1 General Purpose Machine Gun",
            official_designation: "Machine Gun, General Purpose, 7.62mm, C6A1",
            category: "Weapon",
            manufacturer: "FN Herstal",
            country_of_origin: "Belgium",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1985),
            notes: "GPMG based on the FN MAG. Used at platoon / company fire support and as a vehicle-mounted weapon.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C14 Timberwolf Sniper Rifle",
            official_designation: "Rifle, Sniper, 0.338in, C14",
            category: "Weapon",
            manufacturer: "PGW Defence Technologies",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2005),
            notes: "Long-range bolt-action sniper rifle chambered in .338 Lapua Magnum.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C15 Long Range Sniper Weapon",
            official_designation: "Rifle, Sniper, 0.50in, C15",
            category: "Weapon",
            manufacturer: "McMillan Firearms",
            country_of_origin: "United States",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2000),
            notes: "Anti-materiel / long-range sniper weapon chambered in .50 BMG.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C7 Grenade Launcher (M203)",
            official_designation: "Grenade Launcher, 40mm, C16",
            category: "Weapon",
            manufacturer: "Colt Canada",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1995),
            notes: "Under-barrel grenade launcher for C7/C8 family. Fires 40×46mm low-velocity grenades.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "Browning Hi-Power Pistol",
            official_designation: "Pistol, 9mm, Browning Hi-Power",
            category: "Weapon",
            manufacturer: "FN Herstal",
            country_of_origin: "Belgium",
            service_branch: "Joint",
            status: "Legacy",
            introduction_year: Some(1944),
            notes: "Standard service pistol for decades; being replaced by the SIG Sauer P320-M17/M18.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "SIG Sauer P320 Pistol",
            official_designation: "Pistol, 9mm, P320-M17",
            category: "Weapon",
            manufacturer: "SIG Sauer",
            country_of_origin: "Germany/United States",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2022),
            notes: "Modular striker-fired pistol replacing the Browning Hi-Power across the CAF.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "M72 LAW",
            official_designation: "Rocket Launcher, 66mm, M72",
            category: "Weapon",
            manufacturer: "Nammo Talley",
            country_of_origin: "United States",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1970),
            notes: "Single-shot, disposable anti-armour rocket. Widely distributed to infantry sections.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "Carl Gustaf Recoilless Rifle",
            official_designation: "Recoilless Rifle, 84mm, M3 Carl Gustaf",
            category: "Weapon",
            manufacturer: "Saab Bofors Dynamics",
            country_of_origin: "Sweden",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1991),
            notes: "Multi-role, man-portable recoilless rifle. Fires a variety of 84mm rounds including anti-armour, HE, and illumination.",
        },
        // ── Optics ───────────────────────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C79 Optical Sight",
            official_designation: "Sight, Optical, C79A2",
            category: "Optic",
            manufacturer: "Elcan Optical Technologies",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1987),
            notes: "3.4× magnification optical sight for the C7/C8 family. NATO STANAG 4694 rail-compatible.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "Elcan Specter DR",
            official_designation: "Sight, Optical, Specter DR",
            category: "Optic",
            manufacturer: "Elcan Optical Technologies",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2006),
            notes: "Dual-role 1× / 4× optical sight. Provides rapid switch between close-quarters and medium-range engagement.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "AN/PVS-14 Night Vision Monocular",
            official_designation: "Monocular, Night Vision, AN/PVS-14",
            category: "Optic",
            manufacturer: "L3Harris Technologies",
            country_of_origin: "United States",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2000),
            notes: "Gen III image-intensified NVG monocular. Can be head-mounted or weapon-mounted.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "AN/PEQ-2 Laser Aiming Device",
            official_designation: "Target Pointer/Illuminator/Aiming Laser, AN/PEQ-2",
            category: "Optic",
            manufacturer: "L3Harris Technologies",
            country_of_origin: "United States",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1999),
            notes: "Infrared and visible laser aiming device for use with NVG or unaided. Rail-mounted on C7/C8 family.",
        },
        // ── Vehicles ─────────────────────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "LSVW (G-Wagon)",
            official_designation: "Vehicle, Light Support, Wheeled (LSVW)",
            category: "Vehicle",
            manufacturer: "Mercedes-Benz (Daimler-Chrysler)",
            country_of_origin: "Germany",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(1993),
            notes: "Light utility / patrol vehicle. Variants include soft-top, hard-top, and ambulance. 4×4.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "LAV 6.0",
            official_designation: "Vehicle, Armoured, Light, LAV 6.0",
            category: "Vehicle",
            manufacturer: "General Dynamics Land Systems – Canada",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2017),
            notes: "8×8 wheeled armoured vehicle. Primary fighting vehicle for Royal Canadian Armoured Corps reconnaissance and mechanized infantry.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "Leopard 2A4 / 2A6M CAN",
            official_designation: "Tank, Main Battle, Leopard 2",
            category: "Vehicle",
            manufacturer: "Krauss-Maffei Wegmann",
            country_of_origin: "Germany",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2007),
            notes: "120mm smoothbore-armed MBT. CAF operates A4 and upgraded A6M CAN variants.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "M113 APC",
            official_designation: "Carrier, Personnel, Armoured, M113",
            category: "Vehicle",
            manufacturer: "BAE Systems",
            country_of_origin: "United States",
            service_branch: "Army",
            status: "Legacy",
            introduction_year: Some(1964),
            notes: "Tracked APC; still operated in limited numbers by reserve and training units.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "RG-31 Nyala MRAP",
            official_designation: "Vehicle, Patrol, Protected, RG-31 Nyala",
            category: "Vehicle",
            manufacturer: "BAE Systems / Land Systems OMC",
            country_of_origin: "South Africa",
            service_branch: "Army",
            status: "Limited",
            introduction_year: Some(2006),
            notes: "Mine-resistant ambush-protected patrol vehicle used during Afghanistan operations.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "MLVW",
            official_designation: "Vehicle, Logistics, Medium, Wheeled (MLVW)",
            category: "Vehicle",
            manufacturer: "Mercedes-Benz",
            country_of_origin: "Germany",
            service_branch: "Army",
            status: "Legacy",
            introduction_year: Some(1982),
            notes: "2.5-tonne utility truck. Being replaced by the MILCOTS fleet.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "M777 Howitzer",
            official_designation: "Howitzer, 155mm, M777",
            category: "Vehicle",
            manufacturer: "BAE Systems",
            country_of_origin: "United Kingdom/United States",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2005),
            notes: "Lightweight 155mm towed howitzer. Air-transportable by CH-147F Chinook.",
        },
        // ── Communications ───────────────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "PRC-152A Handheld Radio",
            official_designation: "Radio Set, Manpack, AN/PRC-152A",
            category: "Comms",
            manufacturer: "Harris Corporation (L3Harris)",
            country_of_origin: "United States",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2010),
            notes: "Multi-band tactical handheld radio with wideband networking capability.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "PRC-117G Manpack Radio",
            official_designation: "Radio Set, Manpack, AN/PRC-117G",
            category: "Comms",
            manufacturer: "Harris Corporation (L3Harris)",
            country_of_origin: "United States",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2010),
            notes: "Wideband manpack radio with SATCOM capability. Supports SINCGARS and HF.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "VRC-99 Vehicle Radio",
            official_designation: "Radio Set, Vehicular, AN/VRC-99",
            category: "Comms",
            manufacturer: "Harris Corporation (L3Harris)",
            country_of_origin: "United States",
            service_branch: "Army",
            status: "In Service",
            introduction_year: Some(2008),
            notes: "Vehicle-mounted multi-band radio for armoured and wheeled platforms.",
        },
        // ── Personal Kit / Clothing ───────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C2 Composite Body Armour",
            official_designation: "Armour, Body, Composite, C2",
            category: "Clothing",
            manufacturer: "Various (DHC contract)",
            country_of_origin: "Canada",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2006),
            notes: "Modular plate-carrier system with SAPI plates. Replaces the older IBAS.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "CADPAT TW Uniform",
            official_designation: "Uniform, Disruptive Pattern, CADPAT (Temperate Woodland)",
            category: "Clothing",
            manufacturer: "Various (DHC contract)",
            country_of_origin: "Canada",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2002),
            notes: "Standard combat uniform in CADPAT Temperate Woodland pattern. First digitally generated camouflage pattern in NATO service.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "CADPAT AR Uniform",
            official_designation: "Uniform, Disruptive Pattern, CADPAT (Arid Regions)",
            category: "Clothing",
            manufacturer: "Various (DHC contract)",
            country_of_origin: "Canada",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2002),
            notes: "CADPAT Arid Regions pattern for desert and semi-arid operations.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "CF98 Combat Boots",
            official_designation: "Boots, Combat, CF98",
            category: "Clothing",
            manufacturer: "Various (DHC contract)",
            country_of_origin: "Canada",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(1998),
            notes: "Standard issue leather/nylon combat boot.",
        },
        // ── Tools / CES ──────────────────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C8 Cleaning Kit",
            official_designation: "Kit, Cleaning, 5.56mm Rifle/Carbine",
            category: "CES",
            manufacturer: "Various",
            country_of_origin: "Canada",
            service_branch: "Army",
            status: "In Service",
            introduction_year: None,
            notes: "Complete equipment schedule cleaning kit for C7/C8 family weapons.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "C6 Cleaning / Spare Parts Kit",
            official_designation: "Kit, Cleaning and Spare Parts, 7.62mm GPMG",
            category: "CES",
            manufacturer: "Various",
            country_of_origin: "Belgium",
            service_branch: "Army",
            status: "In Service",
            introduction_year: None,
            notes: "CES for the C6 GPMG; includes spare bolt, barrel, and cleaning tools.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "ILBE Rucksack",
            official_designation: "Pack, Assault, ILBE",
            category: "CES",
            manufacturer: "Arc'teryx LEAF / Various",
            country_of_origin: "Canada",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(2010),
            notes: "Integrated Load Bearing Equipment assault rucksack. Part of the CADPAT personal kit system.",
        },
        // ── Ammunition ───────────────────────────────────────────────────────
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "5.56×45mm Ball Ammunition (C77)",
            official_designation: "Cartridge, 5.56mm Ball, C77",
            category: "Ammunition",
            manufacturer: "Various NATO suppliers",
            country_of_origin: "Canada/NATO",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(1985),
            notes: "Standard 5.56×45mm NATO ball ammunition for C7/C8/C9 family.",
        },
        EquipSeed {
            id: uuid::Uuid::new_v4().to_string(),
            common_name: "7.62×51mm Ball Ammunition",
            official_designation: "Cartridge, 7.62mm Ball, C21",
            category: "Ammunition",
            manufacturer: "Various NATO suppliers",
            country_of_origin: "Canada/NATO",
            service_branch: "Joint",
            status: "In Service",
            introduction_year: Some(1960),
            notes: "Standard 7.62×51mm NATO ball for C6 GPMG and C14 (not for sniper use).",
        },
    ];

    for eq in &items {
        conn.execute(
            "INSERT INTO equipment_items
             (id, common_name, official_designation, equipment_category, manufacturer,
              country_of_origin, service_branch, status, introduction_year, notes, active)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,1)",
            rusqlite::params![
                eq.id,
                eq.common_name,
                eq.official_designation,
                eq.category,
                eq.manufacturer,
                eq.country_of_origin,
                eq.service_branch,
                eq.status,
                eq.introduction_year,
                eq.notes,
            ],
        )?;
    }

    // ── Variants ─────────────────────────────────────────────────────────────
    // For each item we insert at least a "base" variant plus common configurations.
    struct VarSeed {
        id: String,
        equipment_id: String,
        variant_name: &'static str,
        calibre_or_spec: Option<&'static str>,
        compatible_accessories: Option<&'static str>,
        notes: &'static str,
    }

    // Build a map of common_name → equipment_id for easy lookup.
    let find_id = |name: &str| -> String {
        items
            .iter()
            .find(|e| e.common_name == name)
            .map(|e| e.id.clone())
            .unwrap_or_default()
    };

    let variants: Vec<VarSeed> = vec![
        // C7A2 Rifle
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C7A2 Rifle"),
            variant_name: "C7A2 (Base)",
            calibre_or_spec: Some("5.56×45mm NATO"),
            compatible_accessories: Some("C79A2, AN/PEQ-2, AN/PVS-14, C16 UGL, foregrip"),
            notes: "Standard configuration, iron sights. 18.6\" barrel, 40-round magazine.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C7A2 Rifle"),
            variant_name: "C7A2 + C79A2 Optic",
            calibre_or_spec: Some("5.56×45mm NATO"),
            compatible_accessories: Some("AN/PEQ-2, AN/PVS-14, C16 UGL, foregrip"),
            notes: "Standard section commander / rifleman configuration with 3.4× C79A2 optical sight.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C7A2 Rifle"),
            variant_name: "C7A2 + C79A2 + C16 UGL",
            calibre_or_spec: Some("5.56×45mm NATO / 40×46mm HV"),
            compatible_accessories: Some("AN/PEQ-2, AN/PVS-14, foregrip"),
            notes: "Grenadier configuration. C16 (M203) grenade launcher fitted under barrel.",
        },
        // C8A3 Carbine
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C8A3 Carbine"),
            variant_name: "C8A3 (Base)",
            calibre_or_spec: Some("5.56×45mm NATO"),
            compatible_accessories: Some("C79A2, Elcan Specter DR, AN/PEQ-2, AN/PVS-14"),
            notes: "Short-barrel carbine; 14.5\" barrel, collapsible stock.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C8A3 Carbine"),
            variant_name: "C8A3 + Specter DR",
            calibre_or_spec: Some("5.56×45mm NATO"),
            compatible_accessories: Some("AN/PEQ-2, AN/PVS-14, suppressor"),
            notes: "Common SOF/recce configuration with dual-role Elcan Specter DR (1× / 4×).",
        },
        // C9A2 LMG
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C9A2 Light Machine Gun"),
            variant_name: "C9A2 (Base)",
            calibre_or_spec: Some("5.56×45mm NATO"),
            compatible_accessories: Some("C79A2, AN/PVS-14, bipod (integral)"),
            notes: "Section light support. Integral bipod, 200-round drum or belt-fed.",
        },
        // C6A1 GPMG
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C6A1 General Purpose Machine Gun"),
            variant_name: "C6A1 (Dismounted / LMG Role)",
            calibre_or_spec: Some("7.62×51mm NATO"),
            compatible_accessories: Some("Bipod, AN/PVS-14"),
            notes: "Dismounted medium machine gun configuration, shoulder-fired or bipod-mounted.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C6A1 General Purpose Machine Gun"),
            variant_name: "C6A1 (Vehicle-Mounted)",
            calibre_or_spec: Some("7.62×51mm NATO"),
            compatible_accessories: Some("Vehicle pintle mount, T&E mechanism, ammo box"),
            notes: "Vehicle co-axial or pintle-mounted configuration.",
        },
        // C79 Optical Sight
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C79 Optical Sight"),
            variant_name: "C79A2",
            calibre_or_spec: Some("3.4× magnification"),
            compatible_accessories: Some("C7A2, C8A3, C9A2"),
            notes: "Current production variant with integral tritium illumination for reticle.",
        },
        // AN/PVS-14
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("AN/PVS-14 Night Vision Monocular"),
            variant_name: "AN/PVS-14 (Head-Mounted)",
            calibre_or_spec: Some("Gen III, 1× to 3× magnification"),
            compatible_accessories: Some("Rhino arm mount, PASGT/MICH helmet mount, dioptre kit"),
            notes: "Standard head-mount configuration with rhino arm for helmet attachment.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("AN/PVS-14 Night Vision Monocular"),
            variant_name: "AN/PVS-14 (Weapon-Mounted)",
            calibre_or_spec: Some("Gen III"),
            compatible_accessories: Some("Picatinny rail mount, weapon mount adapter"),
            notes: "Weapon-mounted configuration used in conjunction with AN/PEQ-2.",
        },
        // Elcan Specter DR
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("Elcan Specter DR"),
            variant_name: "Specter DR (5.56mm BDC)",
            calibre_or_spec: Some("1× / 4× switchable"),
            compatible_accessories: Some("C8A3, C7A2"),
            notes: "Ballistic drop compensator reticle optimised for 5.56×45mm.",
        },
        // LSVW
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("LSVW (G-Wagon)"),
            variant_name: "LSVW Soft-Top",
            calibre_or_spec: Some("2.9L diesel, 4×4"),
            compatible_accessories: Some("Pintle mount, radio rack, CES"),
            notes: "Open-top general purpose variant.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("LSVW (G-Wagon)"),
            variant_name: "LSVW Hard-Top",
            calibre_or_spec: Some("2.9L diesel, 4×4"),
            compatible_accessories: Some("Pintle mount, radio rack, CES"),
            notes: "Enclosed variant for command, signals, and protection.",
        },
        // LAV 6.0
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("LAV 6.0"),
            variant_name: "LAV 6.0 IFV",
            calibre_or_spec: Some("25mm M242 Bushmaster cannon"),
            compatible_accessories: Some("C6A1 co-ax, commander's sight, FLIR"),
            notes: "Infantry fighting vehicle variant with two-person turret.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("LAV 6.0"),
            variant_name: "LAV 6.0 APC",
            calibre_or_spec: Some("12.7mm M2HB or C6A1"),
            compatible_accessories: Some("Pintle mount, ERA kit"),
            notes: "Armoured personnel carrier variant without turret.",
        },
        // PRC-152A
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("PRC-152A Handheld Radio"),
            variant_name: "AN/PRC-152A (Handheld)",
            calibre_or_spec: Some("30–512 MHz"),
            compatible_accessories: Some("Headset, MBITR antenna, speaker-mic, Pelican case"),
            notes: "Standard configuration for section-level communications.",
        },
        // C2 Body Armour
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C2 Composite Body Armour"),
            variant_name: "C2 (Level III+, TW)",
            calibre_or_spec: Some("SAPI ceramic/polyethylene plates"),
            compatible_accessories: Some("Groin protector, collar, shoulder guards, deltoid"),
            notes: "Full modular configuration with temperate woodland soft armour carrier.",
        },
        VarSeed {
            id: uuid::Uuid::new_v4().to_string(),
            equipment_id: find_id("C2 Composite Body Armour"),
            variant_name: "C2 (Minimum Configuration)",
            calibre_or_spec: Some("Front/rear SAPI plates only"),
            compatible_accessories: Some("Plate carrier only"),
            notes: "Minimum force-protection configuration for lower-threat environments.",
        },
    ];

    for v in &variants {
        conn.execute(
            "INSERT INTO equipment_variants
             (id, equipment_id, variant_name, calibre_or_spec, compatible_accessories, notes)
             VALUES (?1,?2,?3,?4,?5,?6)",
            rusqlite::params![
                v.id,
                v.equipment_id,
                v.variant_name,
                v.calibre_or_spec,
                v.compatible_accessories,
                v.notes,
            ],
        )?;
    }

    Ok(())
}
