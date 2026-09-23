//! A singleton is never soft-deletable, so no column of it may claim the soft-delete
//! marker role. Advising `method(soft_delete = true)`, as the message for other tables
//! does, would lead straight into the singleton rejection.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        pub world_name: String,

        deleted: bool,
    }
}

fn main() {}
