//! Two foreign keys are grouped by the referenced table *name* alone, so two columns
//! naming the same table under different paths end up in one group and contradict each
//! other about which table that is.
//!
//! Like `foreign_keys_with_mismatched_types`, the diagnostic is spanned on the second of
//! the two columns -- here the one naming the referenced table under the other path.

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
        #[foreign_key(
            path = crate::overflow_warehouse,
            table = warehouse,
            column = id,
            on_delete = Delete
        )]
        pub destination_warehouse_id: u64,
    }
}

fn main() {}
