//! Every foreign key declares a strategy for each removal its referenced table performs.
//! `warehouse` is soft-deletable, but the foreign key leaves out `on_soft_delete`, so retiring
//! a warehouse would find no strategy for its shipments.
//!
//! The two errors after the first follow from the same gap: the soft-deletion cascade of
//! `warehouse` calls the two functions an `on_soft_delete` strategy would have generated.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(
        plural_name = warehouses,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,

        pub name: String,

        deleted: bool,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = Delete)]
        pub warehouse_id: u64,
    }
}

fn main() {}
