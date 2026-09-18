//! What every method generator is handed, and what it hands back.
//!
//! [`MethodGenerationContext`] is a plain data carrier: the references a generator needs
//! plus the names it would otherwise re-derive. It must not grow generation methods.
//! [`TableContributions`] is the other direction — what a generator wants recorded on the
//! table, returned rather than written, so reordering two generator calls cannot change the
//! table.

use crate::{
    api::{
        db::table::SpacetimeDBTable,
        dsl::table::{CreateDSLMethodArg, SpacetimeDSLTable},
        rust::table::RustStruct,
    },
    internal::column::InternalColumn,
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::format_ident;
use std::collections::BTreeSet;
use syn::Ident;

/// Everything every method generator needs, under one name.
///
/// The derived names are resolved once here rather than in each generator, so a rule about
/// a generated identifier — `field_name_for_found_value` in particular — has one place that
/// states it.
///
/// This is a plain data carrier. It must not grow generation methods: a generator produces
/// one method whole, and a context that generates would take that back.
pub(in crate::internal) struct MethodGenerationContext<'a> {
    pub spacetimedb_table: &'a SpacetimeDBTable,
    pub spacetimedsl_table: &'a SpacetimeDSLTable,
    pub internal_columns: &'a [InternalColumn],
    pub primary_key_column: &'a InternalColumn,

    pub struct_name: Ident,
    pub singular_table_name: Ident,
    pub singular_table_name_as_string: String,
    pub singular_table_name_pascal_case: String,
    pub plural_table_name: Ident,
    pub primary_key_column_name: Ident,
    pub primary_key_column_name_as_string: String,
    /// The local the generated code binds the row it looked up to.
    pub field_name_for_found_value: Ident,
}

impl<'a> MethodGenerationContext<'a> {
    pub(in crate::internal) fn new(
        rust_struct: &'a RustStruct,
        spacetimedb_table: &'a SpacetimeDBTable,
        spacetimedsl_table: &'a SpacetimeDSLTable,
        internal_columns: &'a [InternalColumn],
        primary_key_column: &'a InternalColumn,
    ) -> MethodGenerationContext<'a> {
        let singular_table_name = spacetimedb_table.singular_name.clone();
        let primary_key_column_name = primary_key_column.rust_field_name.clone();

        MethodGenerationContext {
            spacetimedb_table,
            spacetimedsl_table,
            internal_columns,
            primary_key_column,

            struct_name: rust_struct.name.clone(),
            singular_table_name_as_string: singular_table_name.to_string(),
            singular_table_name_pascal_case: RenameRule::PascalCase
                .apply_to_field(singular_table_name.to_string()),
            plural_table_name: spacetimedsl_table.plural_name.clone(),
            primary_key_column_name_as_string: primary_key_column_name.to_string(),
            field_name_for_found_value: format_ident!("the_same_or_another_{singular_table_name}"),
            singular_table_name,
            primary_key_column_name,
        }
    }
}

/// The generators are named for what they produce and they produce a method; the table
/// state they also need is part of their result rather than a side effect, so reordering
/// two generator calls cannot change the table. `SpacetimeDSLTableMethods::generate`
/// collects these and hands them to the one caller that owns the table.
#[derive(Default)]
pub(in crate::internal) struct TableContributions {
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    pub compile_error_checks: BTreeSet<Ident>,
}

impl TableContributions {
    pub(in crate::internal) fn merge(&mut self, other: TableContributions) {
        if let Some(create_dsl_method_arg) = other.create_dsl_method_arg {
            self.create_dsl_method_arg = Some(create_dsl_method_arg);
        }

        self.compile_error_checks.extend(other.compile_error_checks);
    }

    pub(in crate::internal) fn apply_to(self, spacetimedsl_table: &mut SpacetimeDSLTable) {
        if let Some(create_dsl_method_arg) = self.create_dsl_method_arg {
            spacetimedsl_table.create_dsl_method_arg = Some(create_dsl_method_arg);
        }

        spacetimedsl_table
            .compile_error_checks
            .extend(self.compile_error_checks);
    }
}

/// The wrapper type of the primary key column, which `internal/dsl/column.rs` guarantees
/// exists: it rejects a `#[primary_key]` column that carries neither `#[create_wrapper]`
/// nor `#[use_wrapper(...)]`. The one exception, a singleton's injected `id: u8`, never
/// reaches a caller of this.
pub(in crate::internal) fn primary_key_wrapper_type(
    primary_key_column: &InternalColumn,
) -> TokenStream {
    primary_key_column
        .spacetimedsl_column_wrapper_type
        .as_ref()
        .expect(
            "A primary key column must be accompanied by `#[create_wrapper]` or `#[use_wrapper(crate::path::to::MyIdType)]`",
        )
        .struct_name_or_path_tokens()
}
