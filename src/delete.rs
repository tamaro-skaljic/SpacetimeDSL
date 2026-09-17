use std::fmt::Display;

use crate::error::{OneOrMultiple, SpacetimeDSLError};

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

/// What a cascade returns when it refused: the entries it built before it stopped, and the
/// error a delete hook raised, if one did.
///
/// `Entries` is a `Vec<DeletionResultEntry>` when one row of the referenced table was
/// deleted and a `HashMap<&PrimaryKeyValue, Vec<DeletionResultEntry>>` when several were.
#[derive(Debug)]
pub struct OnDeleteStrategyFailure<Entries> {
    pub entries: Entries,
    pub error_from_hook: Option<Box<SpacetimeDSLError>>,
}

#[derive(Debug)]
pub struct DeletionResult {
    pub table_name: Box<str>,
    pub one_or_multiple: OneOrMultiple,
    pub entries: Vec<DeletionResultEntry>,
    /// The error a delete hook of a referencing table raised while the cascade ran.
    ///
    /// Boxed because `SpacetimeDSLError::ReferenceIntegrityViolation` holds a
    /// `DeletionResult`, so an unboxed field would make both types infinitely sized.
    pub error_from_hook: Option<Box<SpacetimeDSLError>>,
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

impl Display for DeletionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.error_from_hook {
            None => write!(f, "{}", self.to_csv()),
            Some(error_from_hook) => {
                write!(
                    f,
                    "Error from a hook: {error_from_hook}\n\n{}",
                    self.to_csv()
                )
            }
        }
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
