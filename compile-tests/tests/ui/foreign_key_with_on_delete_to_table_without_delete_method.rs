//! A foreign key declares `on_delete` only for a referenced table which deletes rows. The
//! `warehouse` table has `method(delete = false)` and is not soft-deletable, so its rows are
//! never removed, and `on_delete = Delete` describes a deletion that cannot happen.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(
        plural_name = warehouses,
        method(update = false, delete = false),
    )]
    #[spacetimedb::table(
        accessor = warehouse,
        public,
    )]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,

        name: String,
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
