//! `#[referenced_by]` declares what happens to other tables' rows when a row of this table
//! goes away. With `method(delete = false)` and no `method(soft_delete = true)` the DSL
//! generates neither a delete nor a soft delete method, so nothing can ever run those
//! strategies.
//!
//! Rejecting the combination is not only tidiness. `for_referenced_by` also registers one
//! half of the paired `this_compilation_error_occurs_because_...` traits that verify
//! `#[referenced_by]` and `#[foreign_key]` agree across the two tables. Silently skipping
//! it would leave every referencing table importing a trait that was never defined.
//!
//! The `shipment` module is here for the fix: once `warehouse` has a delete method, its
//! `#[referenced_by]` needs the table it names. Until then a rejected `#[dsl]` emits
//! nothing else, so the `shipment` table's expansion misses `WarehouseId` and the items
//! the `warehouse` table would have generated for it, which is where the errors after the
//! first come from.

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
