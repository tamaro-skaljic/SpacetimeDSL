::spacetimedsl::spacetimedsl!();

pub mod record {
    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(accessor = record, public)]
    pub struct Record {
        #[created_at]
        created_at: spacetimedb::Timestamp,
    }
}

fn main() {}
