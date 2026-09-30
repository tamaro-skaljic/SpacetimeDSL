::spacetimedsl::spacetimedsl!();

pub mod ticket {
    #[spacetimedsl::dsl(plural_name = tickets, method(update = true))]
    #[spacetimedb::table(accessor = ticket, public)]
    pub struct Ticket {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        pub title: String,

        #[creation_default(None)]
        modified_at: Option<spacetimedb::Timestamp>,
    }
}

fn main() {}
