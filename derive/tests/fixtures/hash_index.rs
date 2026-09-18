//! Covers `#[index(hash)]` and the multi-column `hash(columns = [...])` form - the third
//! index kind beside btree and direct, and the only one no example module uses.
//!
//! A single-column hash index is extracted onto its column like a btree or a direct one, so
//! its snapshots read like their btree counterparts. A multi-column hash index stays on the
//! table and takes the multi-column path.

#[spacetimedsl::dsl(plural_name = sessions, method(update = true))]
#[spacetimedb::table(
    accessor = session,
    index(accessor = region_and_shard, hash(columns = [region_id, shard_id])),
    public,
)]
pub struct Session {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(hash)]
    pub token: String,

    #[index(hash)]
    #[unique]
    pub device_id: u64,

    pub region_id: u64,

    pub shard_id: u64,
}
