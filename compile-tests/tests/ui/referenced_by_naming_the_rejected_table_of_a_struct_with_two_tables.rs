//! A struct with two tables, of which `archived_record` is rejected. Both reference `owner`.
//! The `#[referenced_by]` of `owner` naming `archived_record` reports that it has to be fixed
//! first, and the one naming the accepted `active_record` reports nothing.
//!
//! The error after the first is that one, on the `#[referenced_by]` naming `archived_record`.

::spacetimedsl::spacetimedsl!();

pub mod owner {
    #[spacetimedsl::dsl(plural_name = owners, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = owner, public)]
    pub struct Owner {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::record, table = active_record)]
        #[referenced_by(path = crate::record, table = archived_record)]
        id: u64,
    }
}

pub mod record {
    #[spacetimedsl::dsl(
        plural_name = active_records,
        table = active_record,
        method(update = false, delete = true)
    )]
    #[spacetimedb::table(accessor = active_record, public)]
    #[spacetimedsl::dsl(table = archived_record, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = archived_record, public)]
    pub struct Record {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::owner::OwnerId)]
        #[foreign_key(path = crate::owner, table = owner, column = id, on_delete = Delete)]
        owner_id: u64,
    }
}

fn main() {}
