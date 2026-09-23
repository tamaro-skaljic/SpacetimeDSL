::spacetimedsl::spacetimedsl!();

pub mod record {
#[spacetimedsl::dsl(plural_name = records, method(update = true))]
#[spacetimedb::table(accessor = record, public)]
pub struct Record {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[set_on_update]
    pub(crate) finished_at: spacetimedb::Timestamp,
}
}

fn main() {}
