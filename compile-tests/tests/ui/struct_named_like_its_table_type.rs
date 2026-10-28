//! `#[dsl]` emits the table type `ServerTable` for the `server` table, which other tables reach
//! it through, so a struct of that name would clash with it.

::spacetimedsl::spacetimedsl!();

pub mod server {
    #[spacetimedsl::dsl(plural_name = servers, method(update = false))]
    #[spacetimedb::table(accessor = server, public)]
    pub struct ServerTable {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
