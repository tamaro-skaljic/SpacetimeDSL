use crate::{ContextType, SpacetimeDSLError};

pub trait AsReducerContext {
    fn as_reducer_context(&self) -> Result<&spacetimedb::ReducerContext, SpacetimeDSLError>;
}

macro_rules! impl_as_reducer_context_err {
    ($context:ident, $variant:ident) => {
        impl AsReducerContext for spacetimedb::$context {
            fn as_reducer_context(
                &self,
            ) -> Result<&spacetimedb::ReducerContext, SpacetimeDSLError> {
                Err(crate::get_err(
                    "A Reducer Context is only accessible inside reducers",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_as_reducer_context_ok {
    ($context:ident) => {
        impl AsReducerContext for spacetimedb::$context {
            fn as_reducer_context(
                &self,
            ) -> Result<&spacetimedb::ReducerContext, SpacetimeDSLError> {
                Ok(self)
            }
        }
    };
}

impl_as_reducer_context_err!(AnonymousViewContext, AnonymousView);

impl_as_reducer_context_ok!(ReducerContext);

impl_as_reducer_context_ok!(TxContext);

impl_as_reducer_context_err!(ViewContext, View);

impl crate::Context for spacetimedb::ReducerContext {}

impl crate::Context for spacetimedb::TxContext {}
