//! A foreign key needs a `#[referenced_by]` naming its table on the referenced table's primary
//! key. `warehouse` names no table, so it would delete rows without knowing that `shipment`
//! references them.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
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
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id)]
        warehouse_id: u64,
    }
}

fn main() {}
