//! Every foreign key declares a strategy for each removal its referenced table performs, and
//! the foreign keys of one table are checked one by one. `destination_warehouse_id` leaves
//! out `on_delete` although `warehouse` has a delete method, and `origin_warehouse_id`
//! setting it does not cover for it: deleting a warehouse would leave the destinations
//! dangling.

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
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = Delete, on_soft_delete = Ignore)]
        pub origin_warehouse_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_soft_delete = Ignore)]
        pub destination_warehouse_id: u64,
    }
}

fn main() {}
