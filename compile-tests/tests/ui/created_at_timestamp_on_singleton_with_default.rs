//! `get_<table>` falls back to `DefaultSingleton::get_default`, which a view context can
//! reach too, and a view context has no timestamp to fill in. The default row can only
//! leave a timestamp column empty, so the column has to be able to hold `None`.

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
