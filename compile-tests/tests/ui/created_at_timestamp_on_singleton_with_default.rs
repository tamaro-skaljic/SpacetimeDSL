//! `get_<table>` falls back to `DefaultSingleton::get_default`, which a view context can
//! reach too, and a view context has no timestamp to fill in. The default row can only
//! leave a timestamp column empty, so the column has to be able to hold `None`.
//!
//! The `DefaultSingleton` implementation is here for the fix: once the column is optional,
//! `get_record` falls back to it. Until then a rejected `#[dsl]` emits nothing else, so
//! the implementation misses the `Record` struct, which is where the errors after the
//! first come from.

::spacetimedsl::spacetimedsl!();

pub mod record {
    #[spacetimedsl::dsl(singleton(with_default), method(update = true))]
    #[spacetimedb::table(accessor = record, public)]
    pub struct Record {
        #[created_at]
        created_at: spacetimedb::Timestamp,
    }

    impl crate::spacetimedsl::DefaultSingleton for Record {
        fn get_default(
            _dsl: &crate::spacetimedsl::ReadOnlyDSL<'_, impl crate::spacetimedsl::ReadContext>,
        ) -> Result<Record, crate::spacetimedsl::SpacetimeDSLError> {
            Ok(Record {
                id: 0,
                created_at: None,
            })
        }
    }
}

fn main() {}
