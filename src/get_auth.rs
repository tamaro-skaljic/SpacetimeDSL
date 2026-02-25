use crate::{ContextType, SpacetimeDSLError};

pub trait GetAuth {
    fn auth(&self) -> Result<&spacetimedb::AuthCtx, SpacetimeDSLError>;
}

macro_rules! impl_get_auth_err {
    ($context:ident, $variant:ident) => {
        impl GetAuth for spacetimedb::$context {
            fn auth(&self) -> Result<&spacetimedb::AuthCtx, SpacetimeDSLError> {
                Err(crate::get_err(
                    "An Auth Context is only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_auth_ok {
    ($context:ident) => {
        impl GetAuth for spacetimedb::$context {
            fn auth(&self) -> Result<&spacetimedb::AuthCtx, SpacetimeDSLError> {
                Ok(self.sender_auth())
            }
        }
    };
}

impl_get_auth_err!(AnonymousViewContext, AnonymousView);

impl_get_auth_ok!(ReducerContext);

impl_get_auth_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_auth_err!(ViewContext, View);
