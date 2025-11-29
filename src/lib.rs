pub use itertools;
pub use spacetimedsl_derive::{SpacetimeDSL, dsl, hook};
use std::fmt::Display;

#[doc(hidden)]
pub mod internal;

pub mod prelude {
    pub use crate::{
        ConnectionId, DSL, DSLContext, ReadContext, ReadOnlyDSL, ReadOnlyDSLContext, Sender,
        SpacetimeDSL, SpacetimeDSLError, Timestamp, Wrapper, WriteContext, dsl, hook,
        itertools::Itertools, read_only_dsl,
    };
}

//region Write DSL

pub trait WriteContext:
    spacetimedb::DbContext<DbView = spacetimedb::Local> + Sender + Timestamp + ConnectionId
{
}

impl WriteContext for spacetimedb::ReducerContext {}
impl WriteContext for spacetimedb::TxContext {}

// impl DbContext (ReducerContext, TxContext) for Local and DbContext (AnonymousViewContext, ViewContext) for LocalReadOnly
pub struct DSL<'a, T: WriteContext> {
    pub(crate) ctx: &'a T,
    pub(crate) db: &'a spacetimedb::Local,
}

pub fn dsl<'a, T: spacetimedb::DbContext<DbView = spacetimedb::Local> + WriteContext>(
    ctx: &'a T,
) -> DSL<'a, T> {
    DSL { ctx, db: ctx.db() }
}

pub trait DSLContext<T: WriteContext> {
    fn dsl(&self) -> &DSL<'_, T>;

    fn ctx(&self) -> &T;

    fn db(&self) -> &spacetimedb::Local;
}

impl<T: WriteContext> DSLContext<T> for DSL<'_, T> {
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

pub trait ReadContext: spacetimedb::DbContext<DbView = spacetimedb::LocalReadOnly> {}

// FIXME: Wait for merge and release of https://github.com/clockworklabs/SpacetimeDB/pull/3787
// impl ReadContext for spacetimedb::AnonymousViewContext {}
// impl ReadContext for spacetimedb::ViewContext {}

pub struct ReadOnlyDSL<'a> {
    pub(crate) ctx: &'a dyn ReadContext,
    pub(crate) db: &'a spacetimedb::LocalReadOnly,
}

pub fn read_only_dsl<'a>(ctx: &'a dyn ReadContext) -> ReadOnlyDSL<'a> {
    ReadOnlyDSL { ctx, db: ctx.db() }
}

pub trait ReadOnlyDSLContext {
    fn dsl(&self) -> &ReadOnlyDSL<'_>;

    fn ctx(&self) -> &dyn ReadContext;

    fn db(&self) -> &spacetimedb::LocalReadOnly;
}

impl ReadOnlyDSLContext for ReadOnlyDSL<'_> {
    fn dsl(&self) -> &ReadOnlyDSL<'_> {
        self
    }

    fn ctx(&self) -> &dyn ReadContext {
        self.ctx
    }

    fn db(&self) -> &spacetimedb::LocalReadOnly {
        self.db
    }
}

//endregion Read-Only DSL

pub trait Sender {
    fn sender(&self) -> spacetimedb::Identity;
}

pub trait Timestamp {
    fn timestamp(&self) -> spacetimedb::Timestamp;
}

pub trait ConnectionId {
    fn connection_id(&self) -> Option<spacetimedb::ConnectionId>;
}

pub struct DSLMethodHooks {}

pub trait Wrapper<WrappedType: Clone + Default, WrapperType>:
    Default + Clone + PartialEq + PartialOrd + spacetimedb::SpacetimeType + Display
{
    fn new(value: WrappedType) -> Self;
    fn value(&self) -> WrappedType;
}

// TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/59 SoftDelete Feature

// Don't forget to copy + paste this enum into `derive_input::api::dsl::foreign_key` if you change it
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum OnDeleteStrategy {
    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the deletion fails with a Reference Integrity Violation Error.
     */
    Error,

    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... it's checked whether any primary key value of rows to delete is referenced in a foreign key with `OnDeleteStrategy::Error`.
     * If true, the deletion fails with a Reference Integrity Violation Error and no other on delete strategy is executed.
     * If false, the on delete strategies of all affected rows are executed.
     */
    Delete,

    /**
     * TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 SetNone
     * Because Option is currently not allowed on primary_key and unique/btree indices this strategy isn't used and implemented yet.
     * Available only for columns with type `Option<T>`.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the value of the foreign key column is set to `None`.
     */
    //SetNone,

    /**
     * Available only for columns with a numeric type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the value of the foreign key column is set to `0`.
     */
    SetZero,

    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... nothing happens, which means the referencing rows will reference a primary key value which doesn't exist anymore.
     * The referential integrity is only enforced while creating a row or if a row is updated and the foreign key column value is changed.
     */
    Ignore,
}

#[derive(Debug)]
pub enum SpacetimeDSLError {
    Error(String),
    NotFoundError {
        table_name: Box<str>,
        column_names_and_row_values: Box<str>,
    },
    UniqueConstraintViolation {
        table_name: Box<str>,
        action: Action,
        error_from: ErrorFrom,
        one_or_multiple: OneOrMultiple,
        column_names_and_row_values: Box<str>,
    },
    AutoIncOverflow {
        table_name: Box<str>,
    },
    ReferenceIntegrityViolation(ReferenceIntegrityViolationError),
}

#[derive(Debug)]
pub enum ReferenceIntegrityViolationError {
    OnCreateOrUpdate {
        table_name: Box<str>,
        create_or_update: Action,
        column_names_and_row_values: Box<str>,
    },
    OnDelete(DeletionResult),
}

#[derive(Debug)]
pub enum Action {
    Create,
    Get,
    Update,
    Delete,
}

#[derive(Debug)]
pub enum ErrorFrom {
    SpacetimeDB,
    SpacetimeDSL,
}

#[derive(Debug)]
pub enum OneOrMultiple {
    One,
    Multiple,
}

#[derive(Debug)]
pub struct DeletionResult {
    pub table_name: Box<str>,
    pub one_or_multiple: OneOrMultiple,
    pub entries: Vec<DeletionResultEntry>,
}

#[derive(Debug)]
pub struct DeletionResultEntry {
    pub table_name: Box<str>,
    pub column_name: Box<str>,
    pub strategy: OnDeleteStrategy,
    pub row_value: Box<str>,
    pub child_entries: Vec<DeletionResultEntry>,
}
