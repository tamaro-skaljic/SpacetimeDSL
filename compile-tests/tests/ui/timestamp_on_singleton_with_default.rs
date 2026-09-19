::spacetimedsl::spacetimedsl!();

pub mod record {
    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(accessor = record, public)]
    pub struct Record {
        value: spacetimedb::Timestamp,
    }
}

fn main() {}
