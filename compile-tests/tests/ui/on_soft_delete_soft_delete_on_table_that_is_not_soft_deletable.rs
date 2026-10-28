//! The same rule through the other field: `on_soft_delete = SoftDelete` also retires the
//! rows of this table, so it also needs this table to have a marker column.
//!
//! The error after the first follows from it: the `#[referenced_by]` of the `warehouse` table
//! reports that the `shipment` table has to be fixed first.

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
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_soft_delete = SoftDelete)]
        pub warehouse_id: u64,
    }
}

fn main() {}
