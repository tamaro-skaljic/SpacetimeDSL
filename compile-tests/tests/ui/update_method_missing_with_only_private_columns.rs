//! `method(update = …)` has to be stated. With only private columns the message suggests
//! `update = false`, and names what else makes a table mutable.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(delete = true))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        name: String,
    }
}

fn main() {}
