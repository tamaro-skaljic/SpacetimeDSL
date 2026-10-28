//! A foreign key references only a table with an accepted `#[spacetimedsl::dsl]`. `warehouse`
//! has none, so nothing pairs it with `shipment`.
//!
//! Beside the error which says so, the checks of `shipment` report that `warehouse` has no
//! table type, which only `#[spacetimedsl::dsl]` emits.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    use {crate::spacetimedsl::prelude::*, std::fmt};

    #[derive(SpacetimeType, Clone, Debug, PartialEq, PartialOrd)]
    pub struct WarehouseId {
        id: u64,
    }

    impl Wrapper<u64> for WarehouseId {
        fn new(id: u64) -> WarehouseId {
            WarehouseId { id }
        }

        fn value(&self) -> u64 {
            self.id
        }
    }

    impl From<&WarehouseId> for WarehouseId {
        fn from(id: &WarehouseId) -> WarehouseId {
            id.clone()
        }
    }

    impl fmt::Display for WarehouseId {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "{}", self.id)
        }
    }

    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        pub id: u64,
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
        #[foreign_key(
            path = crate::warehouse,
            table = warehouse,
            column = id,
            referenced_row_method = false
        )]
        warehouse_id: u64,
    }
}

fn main() {}
