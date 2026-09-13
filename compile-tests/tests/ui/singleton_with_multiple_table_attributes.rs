//! With more than one `#[table]` attribute the generator picks one by heuristics, using
//! the `plural_name` as the hint. A singleton has no `plural_name`, so there is nothing to
//! decide with and the ambiguity is rejected instead of guessed.

::spacetimedsl::spacetimedsl!();

pub mod configuration {
    #[spacetimedsl::dsl(singleton, method(update = true))]
    #[spacetimedb::table(accessor = configuration1, public)]
    #[spacetimedb::table(accessor = configuration2, public)]
    pub struct Configuration {
        pub world_name: String,
    }
}

fn main() {}
