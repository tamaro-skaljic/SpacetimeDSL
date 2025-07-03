pub use itertools;
use spacetimedb::ReducerContext;
pub use spacetimedsl_derive::{SpacetimeDSL, dsl};
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

pub trait Wrapper<WrappedType: Clone + Default, WrapperType>:
    Default + Clone + PartialEq + PartialOrd + spacetimedb::SpacetimeType + Display
{
    fn new(value: WrappedType) -> WrapperType;
    fn value(&self) -> WrappedType;
}

// TODO: New Feature "Soft Deletion" - if a table has a column "deleted: bool" then there is another dsl method which sets the flag to true instead of deleting the row.

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

impl Display for SpacetimeDSLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message: String = String::new();

        let dig_spacetimedb = "Unfortunately SpacetimeDB doesn't provide more information";

        message.push_str(&match self {
            SpacetimeDSLError::Error(error) => error.into(),
            SpacetimeDSLError::NotFoundError {
                table_name,
                column_names_and_row_values
            } => format!("Not Found Error while trying to find a row in the `{table_name}` table with `{column_names_and_row_values}`!"),
            SpacetimeDSLError::UniqueConstraintViolation {
                table_name,
                action,
                error_from,
                one_or_multiple,
                column_names_and_row_values,
            } => {
                let column_names_and_row_values = match error_from {
                    ErrorFrom::SpacetimeDB => format!("! {dig_spacetimedb}, so here are all columns and their values: `{column_names_and_row_values}`."),
                    ErrorFrom::SpacetimeDSL => {
                        let one_or_multiple = match one_or_multiple {
                            OneOrMultiple::One => "",
                            OneOrMultiple::Multiple => " There can be two reasons for this: You are inserting or updating somewhere using spacetimedb::ReducerContext instead of spacetimedsl::DSL or the unique multi-column index feature of SpacetimeDSL is broken.",
                        };
                        format!(" because of `{column_names_and_row_values}`!{one_or_multiple}")
                    },
                };

                format!("Unique Constraint Violation Error while trying to {action} a row in the `{table_name}` table{column_names_and_row_values}")
            }
            SpacetimeDSLError::AutoIncOverflow { table_name } => {
                format!("Auto Inc Overflow Error on the `{table_name}` table! {dig_spacetimedb}.")
            }
            SpacetimeDSLError::ReferenceIntegrityViolation(error) => {
                match error {
                    ReferenceIntegrityViolationError::OnCreateOrUpdate {
                        table_name,
                        create_or_update,
                        column_names_and_row_values
                    } => {
                        let create_or_update = match create_or_update {
                            Action::Get | Action::Delete => panic!("Reference Integrity Violation Error On Create Or Update only allowed while creating or updating a row."),
                            action => action.to_string()
                        };

                        format!("Reference Integrity Violation Error while trying to {create_or_update} a row in the `{table_name}` table because of `{column_names_and_row_values}`!")
                    },
                    ReferenceIntegrityViolationError::OnDelete(deletion_result) => {
                        let one_or_multiple_rows = match deletion_result.one_or_multiple {
                            OneOrMultiple::One => "a row",
                            OneOrMultiple::Multiple => "multiple rows",
                        };

                        format!("Reference Integrity Violation Error while trying to delete {one_or_multiple_rows} in the `{}` table because of:\n\n{}", &deletion_result.table_name, deletion_result.to_csv())
                    },
                }
            }
        });

        write!(f, "{}", message)
    }
}

impl Error for SpacetimeDSLError {}

impl From<SpacetimeDSLError> for String {
    fn from(value: SpacetimeDSLError) -> Self {
        value.to_string()
    }
}

#[derive(Debug)]
pub enum Action {
    Create,
    Get,
    Update,
    Delete,
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Create => write!(f, "create"),
            Action::Get => write!(f, "get"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
        }
    }
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

impl DeletionResultEntry {
    pub fn to_csv(
        &self,
        mut entry_id: u128,
        mut parent_entry_id: u128,
        mut message: String,
    ) -> (u128, String) {
        entry_id += 1;

        let table_name = &self.table_name;
        let column_name = &self.column_name;
        let strategy = &self.strategy;
        let row_value = &self.row_value;

        message.push_str(&format!(
            "{entry_id}, {parent_entry_id}, {table_name}, {column_name}, {strategy}, {row_value}\n"
        ));

        parent_entry_id = entry_id;

        for child_entry in &self.child_entries {
            (entry_id, message) = child_entry.to_csv(entry_id, parent_entry_id, message);
        }

        (entry_id, message)
    }
}

impl fmt::Display for DeletionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_csv())
    }
}

impl DeletionResult {
    pub fn to_csv(&self) -> String {
        let mut message: String = String::new();

        message
            .push_str("entry_id, parent_entry_id, table_name, column_name, strategy, row_value,\n");

        let mut entry_id: u128 = 0;

        for entry in &self.entries {
            (entry_id, message) = entry.to_csv(entry_id, 0, message);
        }

        message
    }
}

#[doc(hidden)]
pub mod internal {
    pub struct DSLInternals;
}
