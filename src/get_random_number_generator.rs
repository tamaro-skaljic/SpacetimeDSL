use crate::{ContextType, SpacetimeDSLError};

pub trait GetRandomNumberGenerator {
    fn random_generator(&self) -> Result<&spacetimedb::StdbRng, SpacetimeDSLError>;
}

macro_rules! impl_get_rng_err {
    ($context:ident, $variant:ident) => {
        impl GetRandomNumberGenerator for spacetimedb::$context {
            fn random_generator(&self) -> Result<&spacetimedb::StdbRng, SpacetimeDSLError> {
                Err(crate::get_err(
                    "The Random Number Generator is only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_rng_ok {
    ($context:ident) => {
        impl GetRandomNumberGenerator for spacetimedb::$context {
            fn random_generator(&self) -> Result<&spacetimedb::StdbRng, SpacetimeDSLError> {
                Ok(&self.rng())
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_rng_err!(AnonymousViewContext, AnonymousView);

impl_get_rng_ok!(ReducerContext);

impl_get_rng_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_rng_err!(ViewContext, View);
