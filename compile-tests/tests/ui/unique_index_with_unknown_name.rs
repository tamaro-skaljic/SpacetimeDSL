//! `unique_index(name = …)` makes a multi-column index declared in `#[table]` unique, so a
//! name no `#[table]` index has, such as a misspelling, would have no effect.

::spacetimedsl::spacetimedsl!();

pub mod module {
    #[spacetimedsl::dsl(
        plural_name = modules,
        method(update = true),
        unique_index(name = databse_and_name)
    )]
    #[spacetimedb::table(
        accessor = module,
        index(accessor = database_and_name, btree(columns = [database_id, name])),
        index(accessor = by_name, btree(columns = [name])),
        public
    )]
    pub struct Module {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub database_id: u64,

        pub name: String,
    }
}

fn main() {}
