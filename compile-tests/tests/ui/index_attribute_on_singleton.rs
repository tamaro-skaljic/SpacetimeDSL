//! A singleton holds exactly one row, so an `#[index]` column could only ever find that
//! one row, which the singleton's own getter already returns.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        #[index(btree)]
        pub seed: u64,
    }
}

fn main() {}
