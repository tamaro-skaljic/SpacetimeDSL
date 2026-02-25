use crate::{ContextType, SpacetimeDSLError};

pub trait AsViewContext {
    fn as_view_context(&self) -> Result<spacetimedb::ViewContext, SpacetimeDSLError>;
}

macro_rules! impl_as_view_context_err {
    ($context:ident, $variant:ident) => {
        impl AsViewContext for spacetimedb::$context {
            fn as_view_context(&self) -> Result<spacetimedb::ViewContext, SpacetimeDSLError> {
                Err(crate::get_err(
                    "A View Context is only accessible inside views and reducers",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_as_view_context_ok {
    ($context:ident) => {
        impl AsViewContext for spacetimedb::$context {
            fn as_view_context(&self) -> Result<spacetimedb::ViewContext, SpacetimeDSLError> {
                Ok(self.as_read_only())
            }
        }
    };
}

impl_as_view_context_err!(AnonymousViewContext, AnonymousView);

impl_as_view_context_ok!(ReducerContext);

impl_as_view_context_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_as_view_context_err!(ViewContext, View);

impl crate::Context<spacetimedb::LocalReadOnly> for spacetimedb::ViewContext {}

impl crate::ReadContext for spacetimedb::ViewContext {}
