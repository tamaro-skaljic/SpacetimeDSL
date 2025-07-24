use std::collections::HashSet;

use syn::Ident;

use super::reference::ReferencingTable;
use crate::api::dsl::{column::SpacetimeDSLColumnMethods, method::SpacetimeDSLMethod};

#[derive(Clone)]
pub struct SpacetimeDSLTable {
    pub plural_name: Ident,
    pub is_mutable: bool,
    pub has_created_at_column: bool,
    pub has_modified_at_column: bool,
    pub referencing_tables: Vec<ReferencingTable>,
    pub compile_error_checks: HashSet<Ident>,
}

#[derive(Clone)]
pub struct SpacetimeDSLTableMethods {
    pub create: SpacetimeDSLMethod,
    pub get_all: SpacetimeDSLMethod,
    pub get_count: SpacetimeDSLMethod,
    pub execute_on_delete_strategies_of_referencing_tables_after_one_row_of_this_table_was_deleted:
        Option<SpacetimeDSLMethod>,
    pub execute_on_delete_strategies_of_referencing_tables_after_multiple_rows_of_this_table_were_deleted:
        Option<SpacetimeDSLMethod>,
    pub execute_on_delete_strategies_of_this_table_after_one_row_of_the_referenced_table_was_deleted:
        Vec<SpacetimeDSLMethod>,
    pub execute_on_delete_strategies_of_this_table_after_multiple_rows_of_the_referenced_table_were_deleted:
        Vec<SpacetimeDSLMethod>,
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
}
