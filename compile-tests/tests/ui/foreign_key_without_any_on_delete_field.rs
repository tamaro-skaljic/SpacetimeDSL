//! A `#[foreign_key]` states what becomes of this table's rows when a row of the
//! referenced table goes away. Naming neither `on_delete` nor `on_soft_delete` states
//! nothing, and the referenced table's cascade would find no strategy to run.

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
