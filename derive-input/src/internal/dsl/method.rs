use crate::{
    api::{
        Column,
        db::{
            column::SpacetimeDBColumn,
            index::{Index, IndexType},
        },
        dsl::{
            column::{
                SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex,
            },
            method::SpacetimeDSLMethod,
            table::{
                OnDeleteStrategiesOfReferencingTables, OnDeleteStrategiesOfTheReferencedTable,
                SingletonKind, SpacetimeDSLTableMethods,
            },
        },
    },
    internal::dsl::one_or_multiple::OneOrMultiple,
};
use std::collections::BTreeMap;

mod context;
mod create;
mod delete;
mod foreign_key;
mod get;
mod hook_call;
mod index;
mod naming;
mod on_delete_strategy;
mod reference_integrity;
mod referenced_by;
mod removal;
mod singleton_table;
mod soft_delete;
mod update;
mod upsert;

pub(in crate::internal) use context::{MethodGenerationContext, TableContributions};

use create::for_create;
use delete::{for_delete_many, for_delete_one};
use foreign_key::for_foreign_key;
use get::{for_get_all, for_get_count, for_get_many, for_get_one};
use index::IndexShape;
use on_delete_strategy::ReferencingTables;
use referenced_by::for_referenced_by;
use singleton_table::{for_singleton_delete, for_singleton_get};
use soft_delete::{for_soft_delete_many, for_soft_delete_one};
use update::for_update;
use upsert::for_singleton_upsert;

/// The update method an index earns, if any.
///
/// SpacetimeDB's `update` lives on the primary key index, and no other index implements
/// `PrimaryKey`, so no other index can carry one. `method(update = false)` suppresses it
/// on top of that.
fn update_method_for(
    shape: &IndexShape,
    context: &MethodGenerationContext,
) -> Option<SpacetimeDSLMethod> {
    match context.spacetimedsl_table.has_update_method && shape.is_primary_key {
        false => None,
        true => Some(for_update(shape, context)),
    }
}

/// Which DSL methods an index earns.
///
/// A non-unique index yields many rows, so it earns `get_many` and `delete_many`. A
/// unique index yields at most one, so it earns `get_one_option` and `delete_one`, plus
/// `update` when it is the primary key. The `method(...)` flags suppress the delete and
/// update methods on top of that.
///
/// Single-column and multi-column indices read this rule from here. It must stay stated
/// once: two copies of it disagreed.
fn column_methods_for(
    index: &Index,
    context: &MethodGenerationContext,
) -> SpacetimeDSLColumnMethods {
    let spacetimedsl_table = context.spacetimedsl_table;

    let shape = IndexShape::of(index, context);

    match index.is_unique {
        false => SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
            get_many: for_get_many(&shape, context),
            delete_many: match spacetimedsl_table.has_delete_method {
                false => None,
                true => Some(for_delete_many(&shape, context)),
            },
            soft_delete_many: match spacetimedsl_table.is_soft_deletable() {
                false => None,
                true => Some(for_soft_delete_many(&shape, context)),
            },
        }),
        true => {
            SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                get_one_option: for_get_one(&shape, context),
                update: update_method_for(&shape, context),
                delete_one: match spacetimedsl_table.has_delete_method {
                    false => None,
                    true => Some(for_delete_one(&shape, context)),
                },
                soft_delete_one: match spacetimedsl_table.is_soft_deletable() {
                    false => None,
                    true => Some(for_soft_delete_one(&shape, context)),
                },
            })
        }
    }
}

impl SpacetimeDSLColumnMethods {
    pub(in crate::internal) fn map(
        context: &MethodGenerationContext,
        spacetimedb_column: &SpacetimeDBColumn,
    ) -> Option<SpacetimeDSLColumnMethods> {
        let index = match &spacetimedb_column.single_column_index {
            None => {
                return None;
            }
            Some(index) => index,
        };

        let spacetimedsl_table = context.spacetimedsl_table;

        // `internal/db/column.rs` rejects `#[index]` and `#[unique]` on a singleton's own
        // columns, so the only index a singleton reaches here with is its injected primary
        // key. Getting and deleting its row take no arguments and look it up by that key,
        // which is a different method body rather than a branch inside one. Updating it is
        // the ordinary update with one statement added, unless the table has a default: then
        // the row may be absent, so writing it is an upsert and a method of its own.
        let delete_one = match spacetimedsl_table.has_delete_method {
            false => None,
            true => Some(for_singleton_delete(context)),
        };

        let methods = match spacetimedsl_table.singleton {
            Some(SingletonKind::WithoutDefault) => {
                SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                    get_one_option: for_singleton_get(context),
                    update: update_method_for(&IndexShape::of(index, context), context),
                    delete_one,
                    // `internal/dsl/soft_delete.rs` rejects a soft-deletable singleton.
                    soft_delete_one: None,
                })
            }
            // `internal.rs` rejects `method(update = false)` on such a table, so the upsert
            // always exists: it is the only method which writes the row.
            Some(SingletonKind::WithDefault) => {
                SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                    get_one_option: for_singleton_get(context),
                    update: Some(for_singleton_upsert(context)),
                    delete_one,
                    // `internal/dsl/soft_delete.rs` rejects a soft-deletable singleton.
                    soft_delete_one: None,
                })
            }
            None => column_methods_for(index, context),
        };

        Some(methods)
    }
}

impl SpacetimeDSLTableMethods {
    pub(in crate::internal) fn generate(
        context: &MethodGenerationContext,
        columns: &[Column],
    ) -> syn::Result<(SpacetimeDSLTableMethods, TableContributions)> {
        let MethodGenerationContext {
            spacetimedb_table,
            spacetimedsl_table,
            primary_key_column,
            ..
        } = context;

        let is_singleton = spacetimedsl_table.is_singleton();

        let mut contributions = TableContributions::default();

        // A table with a default has no create method, and skipping the generator is also
        // what withholds the `Create<Table>` argument struct: the generator is its only
        // source.
        let create = match spacetimedsl_table.singleton_has_default() {
            true => None,
            false => {
                let (create, create_contributions) = for_create(context);
                contributions.merge(create_contributions);

                Some(create)
            }
        };

        // A singleton holds one row, so iterating and counting have nothing to say.
        let get_all = match is_singleton {
            true => None,
            false => Some(for_get_all(context)),
        };

        let get_count = match is_singleton {
            true => None,
            false => Some(for_get_count(context)),
        };

        let on_delete_strategies_of_referencing_tables =
            match spacetimedsl_table.referencing_tables.is_empty() {
                true => None,
                false => {
                    let (after_one_row, after_one_row_contributions) = for_referenced_by(
                        &OneOrMultiple::One,
                        spacetimedb_table,
                        spacetimedsl_table,
                        primary_key_column,
                    );
                    contributions.merge(after_one_row_contributions);

                    let (after_multiple_rows, after_multiple_rows_contributions) =
                        for_referenced_by(
                            &OneOrMultiple::Multiple,
                            spacetimedb_table,
                            spacetimedsl_table,
                            primary_key_column,
                        );
                    contributions.merge(after_multiple_rows_contributions);

                    Some(OnDeleteStrategiesOfReferencingTables {
                        after_one_row_of_this_table_was_deleted: after_one_row,
                        after_multiple_rows_of_this_table_were_deleted: after_multiple_rows,
                    })
                }
            };

        let mut on_delete_strategies_of_this_table = vec![];

        let columns_with_foreign_keys: Vec<&Column> = columns
            .iter()
            .filter(|c| c.spacetimedsl_column.foreign_key.is_some())
            .collect();

        if !columns_with_foreign_keys.is_empty() {
            let mut columns_with_foreign_keys_by_table = BTreeMap::new();

            columns_with_foreign_keys.iter().for_each(|c| {
                let name_of_another_table = &c
                    .spacetimedsl_column
                    .foreign_key
                    .as_ref()
                    .expect("The columns were just filtered to those that have a foreign key")
                    .table_name;

                if !columns_with_foreign_keys_by_table.contains_key(name_of_another_table) {
                    columns_with_foreign_keys_by_table.insert(name_of_another_table, vec![]);
                }

                columns_with_foreign_keys_by_table
                    .get_mut(name_of_another_table)
                    .expect("The entry was inserted above when it was missing")
                    .push(*c);
            });

            for (referenced_table_name, columns_with_foreign_key) in
                columns_with_foreign_keys_by_table
            {
                let referencing_tables = match spacetimedsl_table.referencing_tables.is_empty() {
                    true => ReferencingTables::Absent,
                    false => ReferencingTables::Present,
                };

                let (after_one_row, after_one_row_contributions) = for_foreign_key(
                    &OneOrMultiple::One,
                    referencing_tables,
                    spacetimedb_table,
                    referenced_table_name,
                    &columns_with_foreign_key,
                    primary_key_column,
                    spacetimedsl_table,
                )?;
                contributions.merge(after_one_row_contributions);

                let (after_multiple_rows, after_multiple_rows_contributions) = for_foreign_key(
                    &OneOrMultiple::Multiple,
                    referencing_tables,
                    spacetimedb_table,
                    referenced_table_name,
                    &columns_with_foreign_key,
                    primary_key_column,
                    spacetimedsl_table,
                )?;
                contributions.merge(after_multiple_rows_contributions);

                on_delete_strategies_of_this_table.push(OnDeleteStrategiesOfTheReferencedTable {
                    after_one_row_was_deleted: after_one_row,
                    after_multiple_rows_were_deleted: after_multiple_rows,
                });
            }
        }

        let mut multi_column_indices = vec![];

        for multi_column_index in &spacetimedb_table.multi_column_indices {
            // `internal/db/column.rs` moves every single-column index onto its column, so
            // only genuinely multi-column indices reach here. It stops at the first index
            // per column, though, so a column carrying two single-column indices would
            // leak one into this list. Skip it rather than generate it from the wrong path.
            if !matches!(
                multi_column_index.index_type,
                IndexType::BTreeMultiColumn { .. } | IndexType::HashMultiColumn { .. }
            ) {
                continue;
            }

            multi_column_indices.push(column_methods_for(multi_column_index, context));
        }

        let methods = SpacetimeDSLTableMethods {
            create,
            get_all,
            get_count,
            on_delete_strategies_of_referencing_tables,
            on_delete_strategies_of_this_table,
            multi_column_indices,
        };

        Ok((methods, contributions))
    }
}
