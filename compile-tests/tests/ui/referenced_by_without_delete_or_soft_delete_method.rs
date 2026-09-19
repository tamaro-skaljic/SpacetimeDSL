//! `#[referenced_by]` declares what happens to other tables' rows when a row of this table
//! goes away. With `method(delete = false)` and no `method(soft_delete = true)` the DSL
//! generates neither a delete nor a soft delete method, so nothing can ever run those
//! strategies.
//!
//! Rejecting the combination is not only tidiness. `for_referenced_by` also registers one
//! half of the paired `this_compilation_error_occurs_because_...` traits that verify
//! `#[referenced_by]` and `#[foreign_key]` agree across the two tables. Silently skipping
//! it would leave every referencing table importing a trait that was never defined.

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
        #[create_wrapper]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,

        name: String,
    }
}

fn main() {}
