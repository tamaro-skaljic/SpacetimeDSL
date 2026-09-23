use crate::{ContextType, SpacetimeDSLError};

pub trait NewUUID {
    fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError>;
    fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError>;
}

macro_rules! impl_new_uuid_err {
    ($context:ident, $variant:ident) => {
        impl NewUUID for spacetimedb::$context {
            fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                Err(crate::get_err(
                    "UUIDs are only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }

            fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                Err(crate::get_err(
                    "UUIDs are only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

// The inherent methods are called by path, because `self.new_uuid_v4()` would read as a
// call to this trait. `&TxContext` reaches them through its `Deref` to `ReducerContext`.
macro_rules! impl_new_uuid_ok {
    ($context:ident) => {
        impl NewUUID for spacetimedb::$context {
            fn new_uuid_v4(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                spacetimedb::ReducerContext::new_uuid_v4(self).map_err(|error| {
                    SpacetimeDSLError::Error(format!("Failed to generate a UUID v4: {error}"))
                })
            }

            fn new_uuid_v7(&self) -> Result<spacetimedb::Uuid, SpacetimeDSLError> {
                spacetimedb::ReducerContext::new_uuid_v7(self).map_err(|error| {
                    SpacetimeDSLError::Error(format!("Failed to generate a UUID v7: {error}"))
                })
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_new_uuid_err!(AnonymousViewContext, AnonymousView);

impl_new_uuid_ok!(ReducerContext);

impl_new_uuid_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_new_uuid_err!(ViewContext, View);
