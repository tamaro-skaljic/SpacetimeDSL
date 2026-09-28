//! Columns with each kind of restricted visibility, whose setter and mut getter take over that
//! visibility as written: `pub(crate)`, `pub(super)` and `pub(in path)`.

#[spacetimedsl::dsl(plural_name = gauges, method(update = true))]
#[spacetimedb::table(accessor = gauge, public)]
pub struct Gauge {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub(crate) label: String,

    pub(super) reading: u32,

    pub(in crate::instruments) unit: String,
}
