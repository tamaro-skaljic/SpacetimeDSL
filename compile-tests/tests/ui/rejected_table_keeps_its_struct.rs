//! A rejected `#[dsl]` keeps its struct and the `#[spacetimedb::table]` on it. Code which uses
//! the table's accessor compiles, so the rejection is the only error.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    use spacetimedb::Table;

    #[spacetimedsl::dsl(method(update = false))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }

    pub fn count_things(ctx: &spacetimedb::ReducerContext) -> u64 {
        ctx.db.thing().count()
    }
}

fn main() {}
