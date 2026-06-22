use crate::{ContextType, SpacetimeDSLError};

pub trait AsAnonymousViewContext {
    fn as_anonymous_view_context(
        &self,
    ) -> Result<spacetimedb::AnonymousViewContext, SpacetimeDSLError>;
}

macro_rules! impl_as_anonymous_view_context_err {
    ($context:ident, $variant:ident) => {
        impl AsAnonymousViewContext for spacetimedb::$context {
            fn as_anonymous_view_context(
                &self,
            ) -> Result<spacetimedb::AnonymousViewContext, SpacetimeDSLError> {
                Err(crate::get_err(
                    "An Anonymous View Context is only accessible from a reducer context",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_as_anonymous_view_context_ok {
    ($context:ident) => {
        impl AsAnonymousViewContext for spacetimedb::$context {
            fn as_anonymous_view_context(
                &self,
            ) -> Result<spacetimedb::AnonymousViewContext, SpacetimeDSLError> {
                Ok(self.as_anonymous_read_only())
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_as_anonymous_view_context_err!(AnonymousViewContext, AnonymousView);

impl_as_anonymous_view_context_ok!(ReducerContext);

impl_as_anonymous_view_context_ok!(TxContext);

impl AsAnonymousViewContext for spacetimedb::ViewContext {
    fn as_anonymous_view_context(
        &self,
    ) -> Result<spacetimedb::AnonymousViewContext, SpacetimeDSLError> {
        Ok(self.as_anonymous())
    }
}

impl crate::Context<spacetimedb::LocalReadOnly> for spacetimedb::AnonymousViewContext {}

impl crate::ReadContext for spacetimedb::AnonymousViewContext {}
