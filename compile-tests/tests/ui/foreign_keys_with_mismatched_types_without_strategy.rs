//! Two foreign keys pointing at the same referenced table share its pairing checks, so they
//! have to agree on the type of its primary key, also when they declare no strategy.
//!
//! The diagnostic is spanned on the second of the two columns, the one whose type
//! contradicts the first.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait the `shipment` table would have
//! declared for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = false))]
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
        #[foreign_key(path = self, table = warehouse, column = id)]
        pub origin_warehouse_id: u64,

        #[index(btree)]
        #[use_wrapper(WarehouseId)]
        #[foreign_key(path = self, table = warehouse, column = id)]
        pub destination_warehouse_id: u128,
    }
}

fn main() {}
