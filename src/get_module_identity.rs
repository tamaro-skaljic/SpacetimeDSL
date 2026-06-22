use crate::{ContextType, SpacetimeDSLError};

pub trait GetModuleIdentity {
    fn module_identity(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError>;
}

macro_rules! impl_get_identity_err {
    ($context:ident, $variant:ident) => {
        impl GetModuleIdentity for spacetimedb::$context {
            fn module_identity(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError> {
                Err(crate::get_err(
                    "The Module Identity is only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_identity_ok {
    ($context:ident) => {
        impl GetModuleIdentity for spacetimedb::$context {
            fn module_identity(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError> {
                Ok(self.database_identity())
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_identity_err!(AnonymousViewContext, AnonymousView);

impl_get_identity_ok!(ReducerContext);

impl_get_identity_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_identity_err!(ViewContext, View);
