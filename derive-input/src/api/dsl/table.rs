use std::collections::BTreeSet;

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
    pub compile_error_checks: BTreeSet<Ident>,
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    pub hooks: SpacetimeDSLMethodHooks,
}

#[derive(Clone)]
pub struct CreateDSLMethodArg {
    pub struct_name: Ident,
    pub struct_members: Vec<SpacetimeDSLArg>,
    pub struct_impl: TokenStream,
}

/// The two cascade entry points a table earns when another table references it.
///
/// Both exist or neither does: a referenced table needs the one-row and the many-row entry
/// point, because a referencing table's foreign key does not know which delete method will
/// reach it.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfReferencingTables {
    pub after_one_row_of_this_table_was_deleted: SpacetimeDSLMethod,
    pub after_multiple_rows_of_this_table_were_deleted: SpacetimeDSLMethod,
}

/// The two strategy implementations a table earns for one table it references.
///
/// One pair per referenced table, which is why `SpacetimeDSLTableMethods` holds a `Vec` of
/// these rather than two parallel `Vec`s that could go out of step.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfTheReferencedTable {
    pub after_one_row_was_deleted: SpacetimeDSLMethod,
    pub after_multiple_rows_were_deleted: SpacetimeDSLMethod,
}

#[derive(Clone)]
pub struct SpacetimeDSLTableMethods {
    pub create: SpacetimeDSLMethod,
    pub get_all: Option<SpacetimeDSLMethod>,
    pub get_count: Option<SpacetimeDSLMethod>,
    pub on_delete_strategies_of_referencing_tables: Option<OnDeleteStrategiesOfReferencingTables>,
    pub on_delete_strategies_of_this_table: Vec<OnDeleteStrategiesOfTheReferencedTable>,
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
}
