//! Naming the same index twice in `unique_index(name = …)` says nothing the first one did
//! not, so it is most likely a copy that should have named another index.

::spacetimedsl::spacetimedsl!();

pub mod module {
    #[spacetimedsl::dsl(
        plural_name = modules,
        method(update = true),
        unique_index(name = database_and_name),
        unique_index(name = database_and_name)
    )]
    #[spacetimedb::table(
        accessor = module,
        index(accessor = database_and_name, btree(columns = [database_id, name])),
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
