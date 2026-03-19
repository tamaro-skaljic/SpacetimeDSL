use std::collections::HashSet;

use proc_macro2::TokenStream;
use syn::Ident;

use super::reference::ReferencingTable;
use crate::api::dsl::{
    column::SpacetimeDSLColumnMethods,
    hook::SpacetimeDSLMethodHooks,
    method::{SpacetimeDSLArg, SpacetimeDSLMethod},
};

#[derive(Clone)]
pub struct SpacetimeDSLTable {
    pub is_singleton: bool,
    pub plural_name: Ident,
    pub has_update_method: bool,
    pub has_delete_method: bool,
    pub on_insert_set_current_timestamp_column_name: Option<Ident>,
    pub on_update_set_current_timestamp_column_name: Option<Ident>,
    pub referencing_tables: Vec<ReferencingTable>,
    pub compile_error_checks: HashSet<Ident>,
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    pub hooks: SpacetimeDSLMethodHooks,
}

#[derive(Clone)]
pub struct CreateDSLMethodArg {
    pub struct_name: Ident,
    pub struct_members: Vec<SpacetimeDSLArg>,
    pub struct_impl: TokenStream,
}

#[derive(Clone)]
pub struct SpacetimeDSLTableMethods {
    pub create: SpacetimeDSLMethod,
    pub get_all: Option<SpacetimeDSLMethod>,
    pub get_count: Option<SpacetimeDSLMethod>,
    /// For singletons: get_{singular_name}() -> Result<T, Error>
    pub get_singleton: Option<SpacetimeDSLMethod>,
    /// For singletons: update_{singular_name}(entity) -> Result<T, Error>
    pub update_singleton: Option<SpacetimeDSLMethod>,
    /// For singletons: delete_{singular_name}() -> Result<DeletionResult, Error>
    pub delete_singleton: Option<SpacetimeDSLMethod>,
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
