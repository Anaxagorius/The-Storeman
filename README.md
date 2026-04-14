# StoremanPro

**Stand-Alone Inventory Management Application for the Canadian Army Storeman**

> UNCLASSIFIED — Inventory Metadata Only

StoremanPro is an offline-first, single-binary desktop application built in Rust for the Canadian Army Storeman function. It enables fast, accurate receiving, issuing, returning, transferring, stocktaking, and reporting of unit-held equipment and supplies — with barcode-first workflows, a tamper-evident audit log, and role-based access control.

---

## Features

### Core Inventory
- **Item Master** — descriptions, NSN/part numbers, barcodes, categories, unit of issue
- **Location System** — warehouse → aisle → rack → bin hierarchy
- **Stock Balances** — per location and condition code (Serviceable, Unserviceable, Repair, etc.)
- **Low-Stock Alerts** — configurable reorder points with dashboard alerts

### Transactions
| Transaction | Description |
|-------------|-------------|
| **Receive** | Scan-in materiel; auto-create lots/serials; attach reference |
| **Issue** | Barcode issue to named recipient; creates custody record |
| **Return** | Close active custody; update condition; route to stock or quarantine |
| **Transfer** | Bin-to-bin moves; sub-account transfers |
| **Adjust** | Approval-flagged inventory adjustments with reason codes |
| **Dispose** | Controlled disposal workflow with full documentation |
| **Stocktake** | Cycle count; variance calculation; discrepancy reporting |

### Consumable Stores
- Lot/batch tracking with expiry dates
- FEFO (First-Expire-First-Out) guidance
- 30/60/90-day expiry alerts on the dashboard

### Serialized Equipment
- Serial number capture (scan or manual)
- Condition history and inspection reminders
- Per-serial custody tracking

### Controlled Categories
- Configurable governance profiles (Controlled Goods, ITAR, COMSEC metadata, custom)
- Role-based access: view, transact, export, administer
- Dual-approval flags for adjustments/disposals
- Restricted export and PDF watermark support

### Security & Audit
- **Encryption-at-rest** via SQLite WAL journaling
- **Argon2** password hashing
- **Hash-chained tamper-evident audit log** (SHA-256 chain) — detects any retroactive modification
- **Role-Based Access Control** (Storeman, CQMS, Officer, Inspector, Admin)

### Reports & Export
| Report | Export |
|--------|--------|
| On-Hand by Location/Category | CSV |
| Custody List (who signed for what) | CSV |
| Expiring Consumables (30/60/90 days) | CSV |
| Transaction Log | CSV |
| Stocktake Variance | CSV |

---

## Screenshots / UI Screens

| Screen | Description |
|--------|-------------|
| **Login** | Username/password — press Enter to confirm |
| **Dashboard** | Alert cards (expiry, low stock), stat counters, recent transactions |
| **Items** | Searchable item list; add/edit items; view balances, serials, lots |
| **Receive** | Barcode-first receive wizard |
| **Issue** | Quick issue panel — scan, select recipient, confirm |
| **Returns** | Close active custody records with condition update |
| **Stocktake** | Scan-and-count; instant variance report |
| **Reports** | Select template, set date range, export CSV |
| **Admin** | User management, location management |

---

## Getting Started

### Prerequisites
- Rust stable toolchain (`rustup` / [rustup.rs](https://rustup.rs))
- Linux (X11/Wayland), Windows, or macOS

### Build

```bash
git clone https://github.com/Anaxagorius/The-Storeman.git
cd The-Storeman
cargo build --release
```

The binary will be at `target/release/storeman`.

### Run

```bash
./target/release/storeman
```

The database (`storeman.db`) is created automatically in the working directory on first launch.

### Default Credentials

| Username | Password | Role |
|----------|----------|------|
| `admin`  | `admin`  | Admin |

**Change the default password immediately after first login via Admin → Users.**

---

## Architecture

```
The-Storeman/
├── Cargo.toml                  # Workspace
└── crates/
    ├── storeman-core/          # Domain logic, database, transactions
    │   └── src/
    │       ├── models/         # Item, Location, Balance, Serial, Lot, Custody,
    │       │                   #   Transaction, User, ControlledProfile
    │       ├── db/             # SQLite + migrations (WAL, FK enforcement)
    │       ├── auth/           # Argon2 hashing, RBAC guards
    │       ├── audit/          # SHA-256 hash-chained audit log
    │       ├── transactions/   # Receive, Issue, Return, Transfer, Adjust,
    │       │                   #   Dispose, Stocktake engines
    │       └── reports/        # CSV export templates
    └── storeman-app/           # egui/eframe desktop UI
        └── src/
            └── ui/
                ├── app.rs      # Root app + sidebar
                ├── theme.rs    # Canadian Army colour palette
                └── screens/    # One file per screen
```

### Technology Stack
| Component | Crate |
|-----------|-------|
| UI framework | `egui` + `eframe` 0.27 |
| Database | `rusqlite` (SQLite, bundled) |
| Password hashing | `argon2` |
| Audit chain | `sha2` + `hex` |
| Serialization | `serde` + `serde_json` |
| CSV export | `csv` |
| UUIDs | `uuid` v4 |
| Timestamps | `chrono` |

---

## Data Model

```
Item ──────────┬── StockBalance (item × location × condition)
               ├── SerialRecord (serial no., condition, custodian)
               └── LotRecord    (lot/batch, expiry, quantity)

Transaction ───── records every state change with user, timestamp, reason

CustodyRecord ─── who holds what, when issued, when returned

AuditEntry ────── hash-chained log: each entry includes SHA-256 of
                  (previous_hash | payload)
```

---

## Roles & Permissions

| Role | Transact | Approve | Export | View Controlled | Admin |
|------|----------|---------|--------|-----------------|-------|
| Storeman | ✅ | ❌ | ❌ | ❌ | ❌ |
| CQMS | ✅ | ✅ | ✅ | ✅ | ❌ |
| Officer | ❌ | ❌ | ✅ | ❌ | ❌ |
| Inspector | ❌ | ❌ | ✅ | ✅ | ❌ |
| Admin | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Implementation Status

This repository reflects **Phase 2 MVP** of the StoremanPro project proposal:

- [x] **Phase 1** — Architecture, data model, UX design (reflected in this codebase)
- [x] **Phase 2** — SQLite schema + migrations; transaction engine with audit chain; barcode-first receive/issue/return; basic reporting + CSV export
- [ ] **Phase 3** — Full RBAC enforcement in UI; PDF exports; stocktake discrepancy packages; Report Studio drag-drop
- [ ] **Phase 4** — Installer packaging; training quick-start; automated test suite; SOP-aligned report templates

---

## Security Notes

- This application is designed for **UNCLASSIFIED inventory metadata only**.
- Do NOT use this application to store classified information, COMSEC keying material, or protected technical data.
- Refer to the project proposal (`StoremanPro_Project_Proposal_UNCLASSIFIED.docx`) for full governance scope.

---

## License

See [LICENSE](LICENSE).
