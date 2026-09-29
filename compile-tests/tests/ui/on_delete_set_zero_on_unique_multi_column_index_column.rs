//! `SetZero` writes `0` into the foreign key column. On a column of a unique multi-column
//! index, two cleared rows which agree in the index's other columns repeat its values, and
//! SpacetimeDSL checks that index only when a row is created or updated.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(
        plural_name = shipments,
        method(update = true, delete = true),
        unique_index(name = warehouse_and_slot),
    )]
    #[spacetimedb::table(
        accessor = shipment,
        public,
        index(accessor = warehouse_and_slot, btree(columns = [warehouse_id, slot])),
    )]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        pub warehouse_id: u64,

        pub slot: u64,
    }
}

fn main() {}
