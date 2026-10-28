//! A primary key column is usually private. `SetZero` on it is rejected as a primary key
//! column, not as a private one: making the column `pub` would not make `SetZero` work.
//!
//! The error after the first follows from it: the `#[referenced_by]` of the `warehouse` table
//! reports that the `dock` table has to be fixed first.

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
