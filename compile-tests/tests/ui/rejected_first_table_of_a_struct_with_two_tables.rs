//! A struct with two tables, of which the first, `archived_record`, is rejected. The second,
//! `active_record`, is accepted and still emits what the first one would have emitted for both:
//! the wrapper type `RecordId` and the accessors its own methods call.
//!
//! The error after the first follows from it: the `#[referenced_by]` of the `owner` table
//! reports that the `archived_record` table has to be fixed first.

::spacetimedsl::spacetimedsl!();

pub mod owner {
    #[spacetimedsl::dsl(plural_name = owners, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = owner, public)]
    pub struct Owner {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = crate::record, table = archived_record)]
        #[referenced_by(path = crate::record, table = active_record)]
        id: u64,
    }
}

pub mod record {
    #[spacetimedsl::dsl(table = archived_record, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = archived_record, public)]
    #[spacetimedsl::dsl(
        plural_name = active_records,
        table = active_record,
        method(update = false, delete = true)
    )]
    #[spacetimedb::table(accessor = active_record, public)]
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
