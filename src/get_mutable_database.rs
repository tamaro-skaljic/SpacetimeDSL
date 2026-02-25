use crate::{ContextType, SpacetimeDSLError};

pub trait GetMutableDatabase {
    fn mut_db(&self) -> Result<&spacetimedb::Local, SpacetimeDSLError>;
}

macro_rules! impl_get_mut_db_err {
    ($context:ident, $variant:ident) => {
        impl GetMutableDatabase for spacetimedb::$context {
            fn mut_db(&self) -> Result<&spacetimedb::Local, SpacetimeDSLError> {
                Err(crate::get_err(
                    "The Mutable Database is only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_mut_db_ok {
    ($context:ident) => {
        impl GetMutableDatabase for spacetimedb::$context {
            fn mut_db(&self) -> Result<&spacetimedb::Local, SpacetimeDSLError> {
                Ok(&self.db)
            }
        }
    };
}

impl_get_mut_db_err!(AnonymousViewContext, AnonymousView);

impl_get_mut_db_ok!(ReducerContext);

impl_get_mut_db_ok!(TxContext);

impl_get_mut_db_err!(ViewContext, View);
