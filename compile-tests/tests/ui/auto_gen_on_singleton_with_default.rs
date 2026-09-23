::spacetimedsl::spacetimedsl!();

pub mod record {
    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(accessor = record, public)]
    pub struct Record {
        #[create_wrapper]
        #[auto_gen(v4)]
        token: spacetimedb::Uuid,
    }
}

fn main() {}
