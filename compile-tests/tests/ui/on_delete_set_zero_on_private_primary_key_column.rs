//! A primary key column is usually private. `SetZero` on it is rejected as a primary key
//! column, not as a private one: making the column `pub` would not make `SetZero` work.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `dock` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::dock, table = dock)]
        id: u64,
    }
}

pub mod dock {
    #[spacetimedsl::dsl(plural_name = docks, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = dock, public)]
    pub struct Dock {
        #[primary_key]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        warehouse_id: u64,

        pub name: String,
    }
}

fn main() {}
