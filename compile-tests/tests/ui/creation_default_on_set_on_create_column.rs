::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = false))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[creation_default(spacetimedb::Timestamp::UNIX_EPOCH)]
        created_at: spacetimedb::Timestamp,
    }
}

fn main() {}
