use std::{error::Error, fmt::Display};

use crate::{
    Action, DeletionResult, DeletionResultEntry, ErrorFrom, OnDeleteStrategy, OneOrMultiple,
    ReferenceIntegrityViolationError, SpacetimeDSLError,
};

pub struct DSLInternals;

impl crate::Sender for spacetimedb::ReducerContext {
    fn sender(&self) -> spacetimedb::Identity {
        self.sender
    }
}

impl crate::Sender for spacetimedb::TxContext {
    fn sender(&self) -> spacetimedb::Identity {
        self.sender
    }
}

impl crate::Timestamp for spacetimedb::ReducerContext {
    fn timestamp(&self) -> spacetimedb::Timestamp {
        self.timestamp
    }
}

impl crate::Timestamp for spacetimedb::TxContext {
    fn timestamp(&self) -> spacetimedb::Timestamp {
        self.timestamp
    }
}

impl crate::ConnectionId for spacetimedb::ReducerContext {
    fn connection_id(&self) -> Option<spacetimedb::ConnectionId> {
        self.connection_id
    }
}

impl crate::ConnectionId for spacetimedb::TxContext {
    fn connection_id(&self) -> Option<spacetimedb::ConnectionId> {
        self.connection_id
    }
}

impl crate::Sender for spacetimedb::ViewContext {
    fn sender(&self) -> spacetimedb::Identity {
        self.sender
    }
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

impl Display for DeletionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
