::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[unique]
        #[creation_default(String::new())]
        code: String,
    }
}

fn main() {}
