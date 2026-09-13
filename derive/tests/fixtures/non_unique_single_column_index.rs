//! Covers `#[index(btree)]` on a single column: the
//! `SpacetimeDSLColumnMethods::ForIndex` branch, which drives `get_many` and
//! `delete_many` - the plural counterparts of the methods
//! `unique_single_column_index` pins.

#[spacetimedsl::dsl(plural_name = memberships, method(update = true))]
#[spacetimedb::table(accessor = membership, public)]
pub struct Membership {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    pub group_id: u64,
}
