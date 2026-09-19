//! Covers a `#[foreign_key]` column on a singleton table, for both singleton kinds.
//!
//! A singleton may carry a foreign key without an index, and its reference-integrity check
//! has to find the row by the injected primary key rather than through a wrapper the
//! injected key does not have. `ServerBinding` pins that for `update_<table>`, and
//! `ActiveTournament` pins the two check sets `upsert_<table>` runs, one per write path.

#[spacetimedsl::dsl(plural_name = regions, method(update = true))]
#[spacetimedb::table(accessor = region, public)]
pub struct Region {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(RegionId)]
    #[referenced_by(path = self, table = server_binding)]
    #[referenced_by(path = self, table = active_tournament)]
    id: u64,

    pub name: String,
}

#[spacetimedsl::dsl(singleton, method(update = true))]
#[spacetimedb::table(accessor = server_binding, public)]
pub struct ServerBinding {
    #[use_wrapper(RegionId)]
    #[foreign_key(path = self, table = region, column = id, on_delete = Delete)]
    pub region_id: u64,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = active_tournament, public)]
pub struct ActiveTournament {
    #[use_wrapper(RegionId)]
    #[foreign_key(path = self, table = region, column = id, on_delete = Delete)]
    pub region_id: u64,

    #[use_wrapper(RegionId)]
    #[foreign_key(path = self, table = region, column = id, on_delete = Error)]
    qualifier_region_id: u64,
}
