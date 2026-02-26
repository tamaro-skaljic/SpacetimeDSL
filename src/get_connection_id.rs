use crate::{ContextType, SpacetimeDSLError};

pub trait GetConnectionId {
    fn connection_id(&self) -> Result<Option<spacetimedb::ConnectionId>, SpacetimeDSLError>;
}

macro_rules! impl_get_connection_id_err {
    ($context:ident, $variant:ident) => {
        impl GetConnectionId for spacetimedb::$context {
            fn connection_id(
                &self,
            ) -> Result<Option<spacetimedb::ConnectionId>, SpacetimeDSLError> {
                Err(crate::get_err(
                    "A Connection ID is only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_connection_id_ok {
    ($context:ident) => {
        impl GetConnectionId for spacetimedb::$context {
            fn connection_id(
                &self,
            ) -> Result<Option<spacetimedb::ConnectionId>, SpacetimeDSLError> {
                Ok(spacetimedb::ReducerContext::connection_id(self))
            }
        }
    };
}

impl_get_connection_id_err!(AnonymousViewContext, AnonymousView);

impl_get_connection_id_ok!(ReducerContext);

impl_get_connection_id_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_connection_id_err!(ViewContext, View);
