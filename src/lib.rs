pub use itertools;
pub use spacetimedsl_derive::{SpacetimeDSL, dsl, hook};
use std::fmt::Display;
pub use {
    delete::{DeletionResult, DeletionResultEntry, OnDeleteStrategy},
    error::{ReferenceIntegrityViolationError, SpacetimeDSLError},
};

pub mod as_anonymous_view_context;
pub mod as_reducer_context;
pub mod as_view_context;
pub mod get_auth;
pub mod get_connection_id;
pub mod get_immutable_database;
pub mod get_module_identity;
pub mod get_mutable_database;
pub mod get_random;
pub mod get_random_number_generator;
pub mod get_sender;
pub mod get_timestamp;

pub mod delete;
pub mod error;

pub mod prelude {
    pub use crate::{
        Context, DSL, DSLContext, ReadContext, ReadOnlyDSL, ReadOnlyDSLContext, SpacetimeDSL,
        Wrapper, WriteContext,
        as_anonymous_view_context::AsAnonymousViewContext,
        as_reducer_context::AsReducerContext,
        as_view_context::AsViewContext,
        delete::{DeletionResult, DeletionResultEntry},
        dsl,
        error::{ReferenceIntegrityViolationError, SpacetimeDSLError},
        get_auth::GetAuth,
        get_connection_id::GetConnectionId,
        get_immutable_database::GetImmutableDatabase,
        get_module_identity::GetModuleIdentity,
        get_mutable_database::GetMutableDatabase,
        get_random::GetRandom,
        get_random_number_generator::GetRandomNumberGenerator,
        get_sender::GetSender,
        get_timestamp::GetTimestamp,
        hook,
        itertools::Itertools,
        read_only_dsl,
    };
}

use prelude::*;

//region Write DSL

pub struct DSL<'a, T: Context<spacetimedb::Local>> {
    pub(crate) ctx: &'a T,
    pub(crate) db: &'a spacetimedb::Local,
}

pub fn dsl<'a, T: Context<spacetimedb::Local>>(ctx: &'a T) -> DSL<'a, T> {
    DSL {
        ctx,
        db: spacetimedb::DbContext::db(ctx),
    }
}

pub trait DSLContext<T: Context<spacetimedb::Local>> {
    fn dsl(&self) -> &DSL<'_, T>;

    fn ctx(&self) -> &T;

    fn db(&self) -> &spacetimedb::Local;
}

impl<T: Context<spacetimedb::Local>> DSLContext<T> for DSL<'_, T> {
    fn dsl(&self) -> &DSL<'_, T> {
        self
    }

    fn ctx(&self) -> &T {
        self.ctx
    }

    fn db(&self) -> &spacetimedb::Local {
        self.db
    }
}

//endregion Write DSL

//region Read-Only DSL

pub struct ReadOnlyDSL<'a, T: Context<spacetimedb::LocalReadOnly>> {
    pub(crate) ctx: &'a T,
    pub(crate) db: &'a spacetimedb::LocalReadOnly,
}

pub fn read_only_dsl<'a, T: Context<spacetimedb::LocalReadOnly>>(ctx: &'a T) -> ReadOnlyDSL<'a, T> {
    ReadOnlyDSL {
        ctx,
        db: spacetimedb::DbContext::db(ctx),
    }
}

pub trait ReadOnlyDSLContext<T: Context<spacetimedb::LocalReadOnly>> {
    fn dsl(&self) -> &ReadOnlyDSL<'_, T>;

    fn ctx(&self) -> &T;

    fn db(&self) -> &spacetimedb::LocalReadOnly;
}

impl<T: Context<spacetimedb::LocalReadOnly>> ReadOnlyDSLContext<T> for ReadOnlyDSL<'_, T> {
    fn dsl(&self) -> &ReadOnlyDSL<'_, T> {
        self
    }

    fn ctx(&self) -> &T {
        self.ctx
    }

    fn db(&self) -> &spacetimedb::LocalReadOnly {
        self.db
    }
}

//endregion Read-Only DSL

pub enum ContextType {
    AnonymousView,
    Reducer,
    Transaction,
    View,
}

fn get_err(msg: &str, ctx: ContextType) -> SpacetimeDSLError {
    crate::SpacetimeDSLError::Error(format!(
        "{msg}. Tried to access from {} Context.",
        match ctx {
            ContextType::AnonymousView => "Anonymous View",
            ContextType::Reducer => "Reducer",
            ContextType::Transaction => "Transaction",
            ContextType::View => "View",
        }
    ))
}
pub trait Context<T>:
    GetAuth
    + GetConnectionId
    + GetImmutableDatabase
    + GetModuleIdentity
    + GetMutableDatabase
    + GetRandom
    + GetRandomNumberGenerator
    + GetSender
    + GetTimestamp
    + AsAnonymousViewContext
    + AsReducerContext
    + AsViewContext
    + spacetimedb::DbContext<DbView = T>
{
}
pub trait WriteContext: Context<spacetimedb::Local> {}

pub trait ReadContext: Context<spacetimedb::LocalReadOnly> {}

pub struct DSLMethodHooks {}

pub trait Wrapper<WrappedType: Clone + Default, WrapperType>:
    Default + Clone + PartialEq + PartialOrd + spacetimedb::SpacetimeType + Display
{
    fn new(value: WrappedType) -> Self;
    fn value(&self) -> WrappedType;
}

#[doc(hidden)]
pub mod internal {
    pub struct DSLInternals;
}
