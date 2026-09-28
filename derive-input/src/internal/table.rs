use std::collections::{BTreeMap, BTreeSet};

use crate::{
    api::{
        Table,
        db::{
            index::{Index, IndexType},
            table::SpacetimeDBTable,
        },
        dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
    },
    internal::{DSLData, dsl::method::MethodGenerationContext},
};
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::{DeriveInput, Ident, ext::IdentExt};

use crate::internal::rust::column::column_type_path;

pub fn try_parse(
    input: &DeriveInput,
    dsl_data: DSLData,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
) -> syn::Result<Table> {
    // Every other column analysis assumes a path type, so an unsupported one is reported
    // before any of them runs.
    for field in &column_args.fields {
        column_type_path(field.ty)?;
    }

    let rust_struct = crate::internal::rust::table::map_struct(input);

    let column_names: Vec<&Ident> = column_args
        .fields
        .iter()
        .map(|field| field.ident.expect("a named field has an identifier"))
        .collect();

    let IndexAssignment {
        single_column_index_by_column,
        multi_column_indices,
    } = assign_indices(table_args, &column_names);

    let dsl_unique_index_names = dsl_unique_index_names(&dsl_data, &multi_column_indices);

    if dsl_data.kind.singleton().is_some() {
        crate::internal::dsl::singleton::reject_multi_column_indices(&multi_column_indices)?;
    }

    let spacetimedb_table =
        SpacetimeDBTable::map(table_args, multi_column_indices, &dsl_unique_index_names)?;

    let mut spacetimedsl_table =
        SpacetimeDSLTable::try_parse(dsl_data, column_args, &table_args.accessor.unraw())?;

    let super::column::AnalysedColumns {
        columns,
        primary_key_column,
        internal_columns,
        internal_primary_key_column,
    } = super::column::try_parse(
        column_args,
        &rust_struct,
        &spacetimedb_table,
        single_column_index_by_column,
        &spacetimedsl_table,
    )?;

    let context = MethodGenerationContext::new(
        &rust_struct,
        &spacetimedb_table,
        &spacetimedsl_table,
        &internal_columns,
        &internal_primary_key_column,
    );

    let (spacetimedsl_methods, contributions) =
        SpacetimeDSLTableMethods::generate(&context, &columns)?;

    contributions.apply_to(&mut spacetimedsl_table);

    Ok(Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        primary_key_column,
        spacetimedsl_methods,
    })
}

/// Which index of `#[table]` belongs to which column.
struct IndexAssignment {
    /// The first single-column index declared on each column: its `#[primary_key]`,
    /// `#[unique]` or `#[index]`, or a one-column `index(...)` in `#[table]`.
    single_column_index_by_column: BTreeMap<Ident, Index>,
    /// Every index no column claimed, in declaration order.
    multi_column_indices: Vec<Index>,
}

/// Assigns each column, in field order, the first single-column index declared on it; every
/// other index stays in declaration order.
fn assign_indices(table_args: &TableArgs, column_names: &[&Ident]) -> IndexAssignment {
    let mut unassigned: Vec<Option<Index>> = table_args
        .indices
        .iter()
        .map(|index| Some(Index::map(index)))
        .collect();

    let mut single_column_index_by_column = BTreeMap::new();

    for &column_name in column_names {
        let position = unassigned.iter().position(|index| {
            index
                .as_ref()
                .and_then(single_column_of)
                .is_some_and(|column| column == column_name)
        });

        if let Some(position) = position {
            let index = unassigned[position]
                .take()
                .expect("the position of an unassigned index was just found");

            single_column_index_by_column.insert(column_name.clone(), index);
        }
    }

    IndexAssignment {
        single_column_index_by_column,
        multi_column_indices: unassigned.into_iter().flatten().collect(),
    }
}

/// The column of a single-column index, `None` for an index over several columns.
fn single_column_of(index: &Index) -> Option<&Ident> {
    match &index.index_type {
        IndexType::BTreeSingleColumn { column }
        | IndexType::HashSingleColumn { column }
        | IndexType::Direct { column } => Some(column),
        IndexType::BTreeMultiColumn { .. } | IndexType::HashMultiColumn { .. } => None,
    }
}

/// The B-tree multi-column indices `#[dsl(unique_index(name = …))]` makes unique.
fn dsl_unique_index_names(dsl_data: &DSLData, multi_column_indices: &[Index]) -> BTreeSet<Ident> {
    multi_column_indices
        .iter()
        .filter(|index| matches!(index.index_type, IndexType::BTreeMultiColumn { .. }))
        .filter(|index| dsl_data.unique_indices.contains(&index.name))
        .map(|index| index.name.clone())
        .collect()
}
