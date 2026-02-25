use crate::{ContextType, SpacetimeDSLError};

pub trait GetImmutableDatabase {
    fn db(&self) -> Result<&spacetimedb::LocalReadOnly, SpacetimeDSLError>;
}

macro_rules! impl_get_db_err {
    ($context:ident, $variant:ident) => {
        impl GetImmutableDatabase for spacetimedb::$context {
            fn db(&self) -> Result<&spacetimedb::LocalReadOnly, SpacetimeDSLError> {
                Err(crate::get_err(
                    "The Immutable Database is only accessible from Anonymous View Contexts and View Contexts. Call `as_read_only()` on a Reducer Context to get a View Context, then you can access the Immutable Database",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_db_ok {
    ($context:ident) => {
        impl GetImmutableDatabase for spacetimedb::$context {
            fn db(&self) -> Result<&spacetimedb::LocalReadOnly, SpacetimeDSLError> {
                Ok(&self.db)
            }
        }
    };
}

impl_get_db_ok!(AnonymousViewContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_db_err!(ReducerContext, Reducer);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_db_err!(TxContext, Reducer);

impl_get_db_ok!(ViewContext);
