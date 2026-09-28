use {
    crate::DeletionResult,
    std::{error::Error, fmt::Display},
};

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
        create_or_update: CreateOrUpdate,
        column_names_and_row_values: Box<str>,
    },
    OnDelete(DeletionResult),
}

/// The write a reference integrity violation on create or update interrupted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateOrUpdate {
    Create,
    Update,
}

impl Display for CreateOrUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreateOrUpdate::Create => write!(f, "create"),
            CreateOrUpdate::Update => write!(f, "update"),
        }
    }
}
#[derive(Debug)]
pub enum Action {
    Create,
    Get,
    Update,
    Delete,
    SoftDelete,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Create => write!(f, "create"),
            Action::Get => write!(f, "get"),
            Action::Update => write!(f, "update"),
            Action::Delete => write!(f, "delete"),
            Action::SoftDelete => write!(f, "soft delete"),
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

impl Display for SpacetimeDSLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let spacetimedb_gives_no_details =
            "Unfortunately SpacetimeDB doesn't provide more information";

        match self {
            SpacetimeDSLError::Error(error) => write!(f, "{error}"),
            SpacetimeDSLError::NotFoundError {
                table_name,
                column_names_and_row_values,
            } => write!(
                f,
                "Not Found Error while trying to find a row in the `{table_name}` table with `{column_names_and_row_values}`!"
            ),
            SpacetimeDSLError::UniqueConstraintViolation {
                table_name,
                action,
                error_from,
                one_or_multiple,
                column_names_and_row_values,
            } => {
                write!(
                    f,
                    "Unique Constraint Violation Error while trying to {action} a row in the `{table_name}` table"
                )?;

                match error_from {
                    ErrorFrom::SpacetimeDB => write!(
                        f,
                        "! {spacetimedb_gives_no_details}, so here are all columns and their values: `{column_names_and_row_values}`."
                    ),
                    ErrorFrom::SpacetimeDSL => {
                        let one_or_multiple = match one_or_multiple {
                            OneOrMultiple::One => "",
                            OneOrMultiple::Multiple => {
                                " There can be two reasons for this: You are inserting or updating somewhere using spacetimedb::ReducerContext instead of spacetimedsl::DSL or the unique multi-column index feature of SpacetimeDSL is broken."
                            }
                        };
                        write!(
                            f,
                            " because of `{column_names_and_row_values}`!{one_or_multiple}"
                        )
                    }
                }
            }
            SpacetimeDSLError::AutoIncOverflow { table_name } => {
                write!(
                    f,
                    "Auto Inc Overflow Error on the `{table_name}` table! {spacetimedb_gives_no_details}."
                )
            }
            SpacetimeDSLError::ReferenceIntegrityViolation(error) => match error {
                ReferenceIntegrityViolationError::OnCreateOrUpdate {
                    table_name,
                    create_or_update,
                    column_names_and_row_values,
                } => write!(
                    f,
                    "Reference Integrity Violation Error while trying to {create_or_update} a row in the `{table_name}` table because of `{column_names_and_row_values}`!"
                ),
                ReferenceIntegrityViolationError::OnDelete(deletion_result) => {
                    let one_or_multiple_rows = match deletion_result.one_or_multiple {
                        OneOrMultiple::One => "a row",
                        OneOrMultiple::Multiple => "multiple rows",
                    };

                    write!(
                        f,
                        "Reference Integrity Violation Error while trying to delete {one_or_multiple_rows} in the `{}` table because of:\n\n{}",
                        &deletion_result.table_name, deletion_result
                    )
                }
            },
        }
    }
}

impl Error for SpacetimeDSLError {}

impl From<SpacetimeDSLError> for String {
    fn from(value: SpacetimeDSLError) -> Self {
        value.to_string()
    }
}
