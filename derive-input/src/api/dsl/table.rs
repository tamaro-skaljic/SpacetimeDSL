use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use syn::Ident;

use super::reference::ReferencingTable;
use crate::api::dsl::{
    column::SpacetimeDSLColumnMethods,
    hook::SpacetimeDSLMethodHooks,
    method::{SpacetimeDSLArg, SpacetimeDSLMethod},
    soft_delete::SoftDeleteMarker,
};

/// How many rows a singleton table holds, which decides which methods it earns.
///
/// `WithoutDefault` is `#[dsl(singleton)]`: at most one row, and `get_<table>` fails while
/// the row is absent. `WithDefault` is `#[dsl(singleton(with_default))]`: exactly one row
/// from the caller's point of view, because `get_<table>` falls back to the default the
/// table's struct supplies through the `DefaultSingleton` trait.
#[derive(Clone, Copy, PartialEq)]
pub enum SingletonKind {
    WithoutDefault,
    WithDefault,
}

#[derive(Clone)]
pub struct SpacetimeDSLTable {
    /// `None` for an ordinary table.
    pub singleton: Option<SingletonKind>,
    pub plural_name: Ident,
    pub has_update_method: bool,
    pub has_delete_method: bool,
    /// The column a soft deletion writes, if the table is soft-deletable.
    ///
    /// `Some` and `#[dsl(method(soft_delete = true))]` imply each other: the parser rejects
    /// either one without the other, so this is the single question every generator asks.
    pub soft_delete_marker: Option<SoftDeleteMarker>,
    pub on_insert_set_current_timestamp_column_name: Option<Ident>,
    pub on_update_set_current_timestamp_column_name: Option<Ident>,
    pub referencing_tables: Vec<ReferencingTable>,
    pub compile_error_checks: BTreeSet<Ident>,
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    pub hooks: SpacetimeDSLMethodHooks,
}

impl SpacetimeDSLTable {
    pub fn is_singleton(&self) -> bool {
        self.singleton.is_some()
    }

    pub fn is_soft_deletable(&self) -> bool {
        self.soft_delete_marker.is_some()
    }

    /// Whether `get_<table>` falls back to `DefaultSingleton::get_default` instead of failing.
    pub fn singleton_has_default(&self) -> bool {
        self.singleton == Some(SingletonKind::WithDefault)
    }
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
    /// `None` on a `SingletonKind::WithDefault` table: its row is written by
    /// `upsert_<table>`, so it has no create method and no `Create<Table>` argument struct.
    pub create: Option<SpacetimeDSLMethod>,
    pub get_all: Option<SpacetimeDSLMethod>,
    pub get_count: Option<SpacetimeDSLMethod>,
    pub on_delete_strategies_of_referencing_tables: Option<OnDeleteStrategiesOfReferencingTables>,
    pub on_delete_strategies_of_this_table: Vec<OnDeleteStrategiesOfTheReferencedTable>,
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
}
