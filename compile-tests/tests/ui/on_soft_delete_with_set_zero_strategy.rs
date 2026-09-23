//! Soft deletion preserves the row it retires. Clearing the foreign key column of the
//! rows which reference it would destroy exactly what the retirement preserved, leaving
//! no way to tell which row they once pointed at.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(
        plural_name = warehouses,
        method(update = true, delete = false, soft_delete = true),
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
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_soft_delete = SetZero)]
        pub warehouse_id: u64,
    }
}

fn main() {}
