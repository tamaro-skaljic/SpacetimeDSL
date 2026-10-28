//! Tables paired across modules have to be visible to each other. The struct of `warehouse` is
//! private to its module, so the `#[foreign_key]` of `shipment` cannot reach its table.
//!
//! The two errors name the two items of `warehouse` which `shipment` reaches it through: its
//! handle and its table type, both as private as its struct.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(
            path = crate::warehouse,
            table = warehouse,
            column = id,
            on_delete = Delete,
            referenced_row_method = false
        )]
        warehouse_id: u64,
    }
}

fn main() {}
