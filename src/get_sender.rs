use crate::{ContextType, SpacetimeDSLError};

pub trait GetSender {
    fn sender(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError>;
}

macro_rules! impl_get_sender_err {
    ($context:ident, $variant:ident) => {
        impl GetSender for spacetimedb::$context {
            fn sender(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError> {
                Err(crate::get_err(
                    "The Sender Identity is not accessible from Anonymous View Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_sender_ok {
    ($context:ident) => {
        impl GetSender for spacetimedb::$context {
            fn sender(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError> {
                Ok(spacetimedb::$context::sender(self))
            }
        }
    };
}

impl_get_sender_err!(AnonymousViewContext, AnonymousView);

impl_get_sender_ok!(ReducerContext);

impl GetSender for spacetimedb::TxContext {
    fn sender(&self) -> Result<spacetimedb::Identity, SpacetimeDSLError> {
        Ok(spacetimedb::ReducerContext::sender(self))
    }
}

impl_get_sender_ok!(ViewContext);
