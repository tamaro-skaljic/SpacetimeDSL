//! Covers `#[disallow(zero)]` on the column shapes it is allowed on: an unsigned integer, a
//! private `Uuid`, a `#[use_wrapper]` foreign key, where it forbids a reference to no row, and
//! an `#[auto_inc]` primary key, which `create_pallet` skips because SpacetimeDB replaces the
//! `0` it writes there. `create_pallet` and `update_pallet_by_id` check the row after their
//! before hooks, `upsert_pallet_limits` on both of its paths, and the setters and the write
//! methods document the rules.

#[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = false))]
#[spacetimedb::table(accessor = warehouse, public)]
pub struct Warehouse {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = pallet)]
    id: u64,
}

#[spacetimedsl::dsl(
    plural_name = pallets,
    method(update = true),
    hook(before(insert, update))
)]
#[spacetimedb::table(accessor = pallet, public)]
pub struct Pallet {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[disallow(zero)]
    id: u64,

    #[disallow(zero)]
    pub quantity: u32,

    #[disallow(zero)]
    tracking_code: spacetimedb::Uuid,

    #[index(btree)]
    #[use_wrapper(WarehouseId)]
    #[foreign_key(path = self, table = warehouse, column = id)]
    #[disallow(zero)]
    pub warehouse_id: u64,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = pallet_limits, public)]
pub struct PalletLimits {
    #[disallow(zero)]
    pub maximum_quantity: u32,
}
