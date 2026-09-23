::spacetimedsl::spacetimedsl!();

pub mod record {
    #[spacetimedsl::dsl(plural_name = records, method(update = false))]
    #[spacetimedb::table(accessor = record, public)]
    pub struct Record {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[create_wrapper]
        #[auto_gen(v5)]
        token: spacetimedb::Uuid,
    }
}

fn main() {}
