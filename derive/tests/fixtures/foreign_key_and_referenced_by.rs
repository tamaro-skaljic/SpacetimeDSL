//! Covers both ends of a relationship in one file: `#[referenced_by]` on the referenced
//! primary key, which makes the referenced table generate the cascade entry points, and
//! `#[foreign_key]` on the referencing columns, which makes the referencing tables
//! generate the strategy implementations.
//!
//! This is also the fixture `expansion_is_deterministic` expands fifty times, because it
//! has the most order-sensitive input: two referencing tables, two foreign keys on one of
//! them, and three different on-delete strategies.

#[spacetimedsl::dsl(plural_name = warehouses, method(update = true))]
#[spacetimedb::table(accessor = warehouse, public)]
pub struct Warehouse {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(WarehouseId)]
    #[referenced_by(path = self, table = shipment)]
    #[referenced_by(path = self, table = inspection)]
    id: u64,

    pub name: String,
}

#[spacetimedsl::dsl(plural_name = shipments, method(update = true))]
#[spacetimedb::table(accessor = shipment, public)]
pub struct Shipment {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(WarehouseId)]
    #[foreign_key(path = self, table = warehouse, column = id, on_delete = Delete)]
    pub origin_warehouse_id: u64,

    #[index(btree)]
    #[use_wrapper(WarehouseId)]
    #[foreign_key(path = self, table = warehouse, column = id, on_delete = Error)]
    pub destination_warehouse_id: u64,
}

#[spacetimedsl::dsl(plural_name = inspections, method(update = true))]
#[spacetimedb::table(accessor = inspection, public)]
pub struct Inspection {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(WarehouseId)]
    #[foreign_key(path = self, table = warehouse, column = id, on_delete = SetZero)]
    pub warehouse_id: u64,
}
