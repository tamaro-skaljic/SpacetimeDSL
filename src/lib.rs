pub use itertools;
use spacetimedb::ReducerContext;
pub use spacetimedsl_derive::{SpacetimeDSL, dsl};
use std::collections::HashMap;
use std::{
    error::Error,
    fmt::{self, Display},
};

pub struct DSL<'a> {
    pub(crate) ctx: &'a ReducerContext,
}

pub fn dsl<'a>(ctx: &'a ReducerContext) -> DSL<'a> {
    DSL { ctx }
}

pub trait DSLContext {
    fn ctx<'a>(&'a self) -> &'a ReducerContext;
}

impl DSLContext for DSL<'_> {
    fn ctx<'a>(&'a self) -> &'a ReducerContext {
        self.ctx
    }
}

pub trait Wrapper<WrappedType: Clone + Default, WrapperType> {
    fn new(value: WrappedType) -> WrapperType;
    fn default() -> WrapperType;
    fn value(&self) -> WrappedType;
}

// TODO: New Feature "Soft Deletion" - if a table has a column "deleted: bool" then there is another dsl method which sets the flag to true instead of deleting the row.

// Don't forget to copy + paste this enum into `derive_input::api::dsl::foreign_key` if you change it
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum OnDeleteStrategy {
    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the deletion fails.
     */
    Error,

    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... it's checked whether any primary key value of rows to delete is referenced in a foreign key with `OnDeleteStrategy::Error`.
     * If true, the deletion fails and no other on delete strategy is executed.
     * If false, the on delete strategies of all affected rows are executed.
     */
    Delete,

    /**
     * TODO: Because Option is currently not allowed on primary_key and unique/btree indices this strategy isn't used and implemented yet.
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

impl Display for OnDeleteStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OnDeleteStrategy::Error => write!(f, "Error"),
            OnDeleteStrategy::Delete => write!(f, "Delete"),
            OnDeleteStrategy::SetZero => write!(f, "SetZero"),
            OnDeleteStrategy::Ignore => write!(f, "Ignore"),
        }
    }
}

#[derive(Debug)]
pub struct DeletionResult {
    pub table_name: Box<str>,
    pub primary_key_values: Vec<PrimaryKeyValue>,
    pub on_delete_strategy_executions: Option<OnDeleteStrategyExecutions>,
}

impl DeletionResult {
    pub fn is_one_deletion(&self) -> bool {
        self.primary_key_values.len().eq(&1)
    }
}

pub type OnDeleteStrategyExecutions =
    HashMap<PrimaryKeyValue, HashMap<OnDeleteStrategy, Vec<(ColumnName, DeletionResult)>>>;

pub type ReferenceIntegrityViolationError = DeletionResult;

type ColumnName = Box<str>;
type PrimaryKeyValue = Box<str>;

#[doc(hidden)]
pub mod internal {
    use crate::{DeletionResult, OnDeleteStrategy, ReferenceIntegrityViolationError};
    use core::fmt;

    pub struct DSLInternals;

    impl fmt::Display for ReferenceIntegrityViolationError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut message: String = String::new();

            message.push_str("Reference Integrity Violation Error: While deleting ");

            let only_one_deletion = self.is_one_deletion();
            if only_one_deletion {
                message.push_str("a row ");
            } else {
                message.push_str("rows ");
            }

            message.push_str(&format!(
                "in the `{}` table, the following reference integrity violations occurred:\n\n",
                &self.table_name
            ));

            // TODO: Create a data structure like this and use this as the DeletionResult instead of the current one
            message.push_str("error_id,table,pk_value,parent_error_id,reason,trace\n");

            let mut error_id: u128 = 1;
            let mut parent_error_id: Box<str> = "".into();

            for (primary_key_value, results_by_strategy) in
                self.on_delete_strategy_executions.as_ref().expect("should exist")
            {
                message.push_str(&format!(
                    "{},{},{},{},,\n",
                    &error_id, &self.table_name, &primary_key_value, &parent_error_id
                ));

                parent_error_id = format!("{error_id}").into();
                let trace = format!("{error_id}").into();

                for (foreign_key_column_name, result) in
                    results_by_strategy.get(&OnDeleteStrategy::Error).expect("should exist")
                {
                    (message, error_id) = process_each_reference_integrity_violation(
                        message,
                        error_id,
                        &parent_error_id,
                        foreign_key_column_name,
                        primary_key_value,
                        &trace,
                        result,
                    )
                }
            }

            write!(f, "{}", message)
        }
    }

    fn process_each_reference_integrity_violation(
        mut message: String,
        mut error_id: u128,
        parent_error_id: &Box<str>,
        foreign_key_column_name: &Box<str>,
        foreign_key_column_value: &Box<str>,
        trace: &Box<str>,
        result: &DeletionResult,
    ) -> (String, u128) {
        for primary_key_value in &result.primary_key_values {
            error_id += 1;

            let trace: Box<str> = format!("{trace}<-{error_id}").into();
            message.push_str(&format!(
                "{},{},{},{},{} == {},{}{}\n",
                &error_id,
                &result.table_name,
                &primary_key_value,
                &parent_error_id,
                foreign_key_column_name,
                foreign_key_column_value,
                trace,
                &error_id
            ));

            if result.on_delete_strategy_executions.is_some() {
                for (foreign_key_column_name, result) in result
                    .on_delete_strategy_executions
                    .as_ref()
                    .expect("should exist")
                    .get(primary_key_value)
                    .expect("should exist")
                    .get(&OnDeleteStrategy::Error)
                    .expect("should exist")
                {
                    (message, error_id) = process_each_reference_integrity_violation(
                        message,
                        error_id,
                        &parent_error_id,
                        foreign_key_column_name,
                        &primary_key_value,
                        &trace,
                        result,
                    )
                }
            }
        }

        (message, error_id)
    }
}
