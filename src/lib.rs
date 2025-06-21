pub use internal::DSLContext;
pub use itertools;
use spacetimedb::ReducerContext;
pub use spacetimedsl_derive::{SpacetimeDSL, dsl};
use std::collections::HashMap;

pub struct DSL<'a> {
    pub(crate) ctx: &'a ReducerContext,
}

pub fn dsl<'a>(ctx: &'a ReducerContext) -> DSL<'a> {
    DSL { ctx }
}

pub trait Wrapper<WrappedType: Clone + Default, WrapperType> {
    fn new(value: WrappedType) -> WrapperType;
    fn default() -> WrapperType;
    fn value(&self) -> WrappedType;
}

pub struct ReferenceIntegrityViolationError {
    pub table: Box<str>,
    pub value: Box<str>,
    pub primary_key_column: Box<str>,
    pub violations: Vec<ReferenceIntegrityViolations>,
}

type ForeignTableName = Box<str>;
type ForeignColumnName = Box<str>;
type ForeignRowPrimaryKeyValue = Box<str>;
type ReferenceIntegrityViolations =
    HashMap<ForeignTableName, HashMap<ForeignColumnName, Vec<ForeignRowPrimaryKeyValue>>>;

#[doc(hidden)]
pub mod internal {
    use crate::ReferenceIntegrityViolationError;
    use core::fmt;
    use spacetimedb::ReducerContext;

    pub struct DSLInternals;

    pub trait DSLContext {
        fn ctx<'a>(&'a self) -> &'a ReducerContext;
    }

    impl DSLContext for crate::DSL<'_> {
        fn ctx<'a>(&'a self) -> &'a ReducerContext {
            self.ctx
        }
    }

    impl fmt::Display for ReferenceIntegrityViolationError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut error_message: String = String::new();

            error_message.push_str(&format!(
                "deletion error on table `{}`: The value `{}` in the primary key column `{}`...",
                self.table, self.value, self.primary_key_column
            ));

            for violation in self.violations.iter() {
                for (foreign_table_name, foreign_row_primary_key_values_by_foreign_column_name) in
                    violation.iter()
                {
                    error_message.push_str(&format!(
                        "\n... is referenced in the table {} by...\n",
                        foreign_table_name
                    ));

                    for (foreign_column_name, foreign_row_primary_key_values) in
                        foreign_row_primary_key_values_by_foreign_column_name.iter()
                    {
                        error_message.push_str(&format!("... the foreign key on the column `{}` with `on_delete = Error`. Found:\n{:?}\n", foreign_column_name, foreign_row_primary_key_values));
                    }
                }
            }

            write!(f, "{}", error_message)
        }
    }
}
