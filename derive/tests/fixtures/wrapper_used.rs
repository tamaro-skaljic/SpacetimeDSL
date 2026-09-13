//! Covers `#[use_wrapper(...)]`, which reuses a wrapper type another table created
//! instead of emitting one. Both accepted spellings appear: a bare identifier and a full
//! path.
//!
//! No wrapper type may be generated here - that is the difference to
//! `wrapper_created_unnamed`.

#[spacetimedsl::dsl(plural_name = shipments, method(update = true))]
#[spacetimedb::table(accessor = shipment, public)]
pub struct Shipment {
    #[primary_key]
    #[use_wrapper(OrderId)]
    id: u64,

    #[unique]
    #[use_wrapper(crate::warehouse::WarehouseId)]
    pub warehouse_id: u64,
}
