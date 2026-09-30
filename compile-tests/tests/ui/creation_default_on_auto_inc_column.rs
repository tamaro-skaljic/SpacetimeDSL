::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[creation_default(1)]
        id: u64,
    }
}

fn main() {}
