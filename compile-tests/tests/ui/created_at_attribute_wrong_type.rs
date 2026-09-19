::spacetimedsl::spacetimedsl!();

pub mod record {
#[spacetimedsl::dsl(plural_name = records, method(update = true))]
#[spacetimedb::table(accessor = record, public)]
pub struct Record {
    #[primary_key]
    #[auto_inc]
    id: u64,

    #[created_at]
    started_at: String,
}
}

fn main() {}
