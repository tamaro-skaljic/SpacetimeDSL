use crate::{ContextType, SpacetimeDSLError};

pub trait GetRandom {
    fn rand<T>(&self) -> Result<T, SpacetimeDSLError>
    where
        spacetimedb::rand::distributions::Standard:
            spacetimedb::rand::distributions::Distribution<T>;
}

macro_rules! impl_get_random_err {
    ($context:ident, $variant:ident) => {
        impl GetRandom for spacetimedb::$context {
            fn rand<T>(&self) -> Result<T, SpacetimeDSLError>
            where
                spacetimedb::rand::distributions::Standard:
                    spacetimedb::rand::distributions::Distribution<T>,
            {
                Err(crate::get_err(
                    "Random Numbers are only accessible from Reducer Contexts",
                    ContextType::$variant,
                ))
            }
        }
    };
}

macro_rules! impl_get_random_ok {
    ($context:ident) => {
        impl GetRandom for spacetimedb::$context {
            fn rand<T>(&self) -> Result<T, SpacetimeDSLError>
            where
                spacetimedb::rand::distributions::Standard:
                    spacetimedb::rand::distributions::Distribution<T>,
            {
                Ok(self.random())
            }
        }
    };
}

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_random_err!(AnonymousViewContext, AnonymousView);

impl_get_random_ok!(ReducerContext);

impl_get_random_ok!(TxContext);

// FIXME: https://github.com/clockworklabs/SpacetimeDB/issues/4439
impl_get_random_err!(ViewContext, View);
