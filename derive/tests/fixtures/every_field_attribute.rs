//! Covers every field attribute SpacetimeDSL reads on accepted tables: `create_wrapper`,
//! `use_wrapper`, `foreign_key`, `referenced_by`, `set_on_create`, `set_on_update`,
//! `set_on_soft_delete`, `auto_gen` and `creation_default`.
//!
//! The harness only expands fixtures and never compiles them, so this fixture cannot notice
//! a field attribute missing from the helper attributes of the `SpacetimeDSL` derive;
//! `helper_attributes_match_field_attributes` checks that.

use spacetimedb::{Timestamp, Uuid};

#[spacetimedsl::dsl(plural_name = owners, method(update = true, delete = true))]
#[spacetimedb::table(accessor = owner, public)]
pub struct Owner {
    #[primary_key]
    #[create_wrapper]
    #[auto_gen(v7)]
    #[referenced_by(path = self, table = gadget)]
    id: Uuid,

    pub name: String,
}

#[spacetimedsl::dsl(
    plural_name = gadgets,
    method(update = true, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = gadget, public)]
pub struct Gadget {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(OwnerId)]
    #[foreign_key(path = self, table = owner, column = id, on_delete = Delete)]
    pub owner_id: Uuid,

    #[creation_default(1)]
    pub rating: u8,

    #[set_on_create]
    created_at: Timestamp,

    #[set_on_update]
    modified_at: Option<Timestamp>,

    #[set_on_soft_delete]
    retired_at: Option<Timestamp>,
}
