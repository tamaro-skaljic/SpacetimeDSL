//! A singleton holds exactly one row, so its generated methods are singular and take
//! their name from the table accessor. A `plural_name` would have nothing to name.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, plural_name = configurations, method(update = true))]
    #[spacetimedb::table(accessor = configuration, public)]
    pub struct Configuration {
        pub world_name: String,
    }
}

fn main() {}
