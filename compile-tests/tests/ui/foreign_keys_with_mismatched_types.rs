//! Two foreign keys pointing at the same referenced table are generated into one cascade
//! method, which collects the referenced primary key values into a single collection. Two
//! different column types cannot go into it.
//!
//! The diagnostic is spanned on the second of the two columns, the one whose type
//! contradicts the first. That is deliberately more precise than the `Span::call_site()`
//! every other diagnostic in this crate carries.

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
