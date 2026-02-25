use crate::{ContextType, SpacetimeDSLError};

pub trait GetTimestamp {
    fn timestamp(&self) -> Result<spacetimedb::Timestamp, SpacetimeDSLError>;
}

macro_rules! impl_get_timestamp_err {
    ($context:ident, $variant:ident) => {
        impl GetTimestamp for spacetimedb::$context {
            fn timestamp(&self) -> Result<spacetimedb::Timestamp, SpacetimeDSLError> {
                Err(crate::get_err(
                    "The Timestamp is only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_timestamp_ok {
    ($context:ident) => {
        impl GetTimestamp for spacetimedb::$context {
            fn timestamp(&self) -> Result<spacetimedb::Timestamp, SpacetimeDSLError> {
                Ok(self.timestamp)
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_timestamp_err!(AnonymousViewContext, AnonymousView);

impl_get_timestamp_ok!(ReducerContext);

impl_get_timestamp_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_timestamp_err!(ViewContext, View);
