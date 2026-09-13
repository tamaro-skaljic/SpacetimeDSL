//! A non-singleton table must name its plural, because every generated method name is
//! built from it.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(method(update = false))]
    #[spacetimedb::table(accessor = thing, public)]
    pub struct Thing {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
