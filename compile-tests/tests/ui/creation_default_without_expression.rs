::spacetimedsl::spacetimedsl!();

pub mod player {
    #[spacetimedsl::dsl(plural_name = players, method(update = true))]
    #[spacetimedb::table(accessor = player, public)]
    pub struct Player {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[creation_default]
        pub coins: u32,
    }
}

fn main() {}
