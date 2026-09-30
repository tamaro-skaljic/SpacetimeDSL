::spacetimedsl::spacetimedsl!();

pub mod session {
    #[spacetimedsl::dsl(plural_name = sessions, method(update = false))]
    #[spacetimedb::table(accessor = session, public)]
    pub struct Session {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[create_wrapper]
        #[auto_gen(v4)]
        #[creation_default(spacetimedb::Uuid::NIL)]
        token: spacetimedb::Uuid,
    }
}

fn main() {}
