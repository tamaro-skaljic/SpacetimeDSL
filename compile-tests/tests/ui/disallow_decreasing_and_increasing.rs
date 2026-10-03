::spacetimedsl::spacetimedsl!();

pub mod item {
    #[spacetimedsl::dsl(plural_name = items, method(update = true))]
    #[spacetimedb::table(accessor = item, public)]
    pub struct Item {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[disallow(decreasing, increasing)]
        pub score: i64,
    }
}

fn main() {}
