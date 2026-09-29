//! A foreign key declares `on_delete` when its referenced table deletes rows, so the cascade
//! finds a strategy for every row it reaches. This one declares no strategy at all, although
//! `warehouse` has a delete method.
//!
//! The two errors after the first follow from the same gap: the deletion cascade of
//! `warehouse` calls the two functions an `on_delete` strategy would have generated.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,

        pub name: String,
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
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id)]
        pub warehouse_id: u64,
    }
}

fn main() {}
