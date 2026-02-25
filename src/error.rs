use std::{error::Error, fmt::Display};

use crate::{DeletionResult, delete::OnDeleteStrategy};

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

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl Display for OnDeleteStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnDeleteStrategy::Error => write!(f, "Error"),
            OnDeleteStrategy::Delete => write!(f, "Delete"),
            OnDeleteStrategy::SetZero => write!(f, "SetZero"),
            OnDeleteStrategy::Ignore => write!(f, "Ignore"),
        }
    }
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

        write!(f, "{message}")
    }
}

impl Error for SpacetimeDSLError {}

impl From<SpacetimeDSLError> for String {
    fn from(value: SpacetimeDSLError) -> Self {
        value.to_string()
    }
}
