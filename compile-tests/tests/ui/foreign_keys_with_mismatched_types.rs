//! Two foreign keys pointing at the same referenced table are generated into one cascade
//! method, which collects the referenced primary key values into a single collection. Two
//! different column types cannot go into it.
//!
//! The diagnostic is spanned on the second of the two columns, the one whose type
//! contradicts the first.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = self, table = shipment)]
        id: u64,
    }

    #[spacetimedsl::dsl(plural_name = shipments, method(update = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(WarehouseId)]
        #[foreign_key(path = self, table = warehouse, column = id, on_delete = Delete)]
        pub origin_warehouse_id: u64,

        #[index(btree)]
        #[use_wrapper(WarehouseId)]
        #[foreign_key(path = self, table = warehouse, column = id, on_delete = Delete)]
        pub destination_warehouse_id: u128,
    }
}

fn main() {}
