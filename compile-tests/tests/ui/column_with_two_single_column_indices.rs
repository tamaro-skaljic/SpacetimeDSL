//! A column gets its lookup methods from one single-column index. A second one on the same
//! column would generate no methods, so it is rejected.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(update = true))]
    #[spacetimedb::table(
        accessor = thing,
        index(accessor = by_name, hash(columns = [name])),
        public
    )]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        pub name: String,
    }
}

fn main() {}
