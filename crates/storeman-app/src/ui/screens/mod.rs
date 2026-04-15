pub mod login;
pub mod dashboard;
pub mod items;
pub mod receive;
pub mod issue;
pub mod returns;
pub mod stocktake;
pub mod reports;
pub mod admin;
pub mod equipment_ref;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Dashboard,
    Items,
    Receive,
    Issue,
    Returns,
    Stocktake,
    Reports,
    Admin,
    EquipmentRef,
}
