//! Soft-deleting a row must not physically remove the rows which reference it. That is
//! what `on_delete = Delete` is for; reaching for it from `on_soft_delete` would turn a
//! reversible retirement into an irreversible cascade.

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
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_soft_delete = Delete)]
        pub warehouse_id: u64,
    }
}

fn main() {}
