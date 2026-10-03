//! Covers two foreign keys to the same table which spell the same type and the same path
//! differently: `u64` and `core::primitive::u64` in `Shipment`, `i64` and
//! `core::primitive::i64` in `Transfer`, `::other_crate::tables` and `other_crate::tables` in
//! both. They are grouped into one cascade function, so the generator
//! checks that the grouped columns agree on the type and the path, and has to accept
//! spellings of the same one.
//!
//! The fixture is only expanded, never compiled, so `other_crate` does not have to exist.

#[spacetimedsl::dsl(plural_name = shipments, method(update = true))]
#[spacetimedb::table(accessor = shipment, public)]
pub struct Shipment {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(::other_crate::tables::WarehouseId)]
    #[foreign_key(path = ::other_crate::tables, table = warehouse, column = id, on_delete = Delete)]
    pub origin_warehouse_id: u64,

    #[index(btree)]
    #[use_wrapper(other_crate::tables::WarehouseId)]
    #[foreign_key(path = other_crate::tables, table = warehouse, column = id, on_delete = Delete)]
    pub destination_warehouse_id: core::primitive::u64,
}

#[spacetimedsl::dsl(plural_name = transfers, method(update = true))]
#[spacetimedb::table(accessor = transfer, public)]
pub struct Transfer {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(::other_crate::tables::LedgerId)]
    #[foreign_key(path = ::other_crate::tables, table = ledger, column = id, on_delete = Delete)]
    pub source_ledger_id: i64,

    #[index(btree)]
    #[use_wrapper(other_crate::tables::LedgerId)]
    #[foreign_key(path = other_crate::tables, table = ledger, column = id, on_delete = Delete)]
    pub target_ledger_id: core::primitive::i64,
}
