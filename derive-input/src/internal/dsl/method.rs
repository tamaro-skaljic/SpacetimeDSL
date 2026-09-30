pub use context::{MethodGenerationContext, TableContributions};
use {
    crate::{
        api::{
            Column,
            db::{column::SpacetimeDBColumn, index::Index},
            dsl::{
                column::{
                    SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                    SpacetimeDSLColumnMethodsForUniqueIndex,
                },
                method::SpacetimeDSLMethod,
                table::{
                    CascadeEntryPoints, OnDeleteStrategiesOfReferencingTables,
                    OnDeleteStrategiesOfTheReferencedTable, SingletonKind, SpacetimeDSLTableKind,
                    SpacetimeDSLTableMethods,
                },
                wrapper::WrapperMethod,
            },
        },
        internal::{column::canonical_type, dsl::one_or_multiple::OneOrMultiple, error},
    },
    create::for_create,
    delete::{for_delete_many, for_delete_one},
    foreign_key::{for_foreign_key, foreign_key_of},
    get::{for_get_all, for_get_count, for_get_many, for_get_one},
    index::IndexShape,
    on_delete_strategy::ReferencingTables,
    referenced_by::for_referenced_by,
    removal::Removal,
    singleton_table::{for_singleton_delete, for_singleton_get},
    soft_delete::{for_soft_delete_many, for_soft_delete_one},
    std::collections::BTreeMap,
    update::for_update,
    upsert::for_singleton_upsert,
    wrapper_method::for_wrapper_methods,
};

mod context;
mod create;
mod delete;
pub mod disallow;
pub mod doc;
mod foreign_key;
mod get;
mod hook_call;
mod index;
mod message;
pub mod naming;
mod on_delete_strategy;
mod pairing;
mod reference_integrity;
mod referenced_by;
pub mod relationship_doc;
mod removal;
mod singleton_table;
mod soft_delete;
mod update;
mod upsert;
mod wrapper_method;

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
    pub(crate) fn map(
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

        let methods = match spacetimedsl_table.kind {
            SpacetimeDSLTableKind::Singleton(SingletonKind::WithoutDefault) => {
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
            SpacetimeDSLTableKind::Singleton(SingletonKind::WithDefault) => {
                SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
                    get_one_option: for_singleton_get(context),
                    update: Some(for_singleton_upsert(context)),
                    delete_one,
                    // `internal/dsl/soft_delete.rs` rejects a soft-deletable singleton.
                    soft_delete_one: None,
                })
            }
            SpacetimeDSLTableKind::Normal { .. } => column_methods_for(index, context),
        };

        Some(methods)
    }
}

impl SpacetimeDSLTableMethods {
    pub(crate) fn generate(
        context: &MethodGenerationContext,
        columns: &[Column],
    ) -> syn::Result<(SpacetimeDSLTableMethods, TableContributions)> {
        let mut contributions = TableContributions::default();

        let (create, get_all, get_count) = table_level_methods(context, &mut contributions);

        let on_delete_strategies_of_referencing_tables = referenced_side_entry_points(context);

        let foreign_key_columns_by_referenced_table =
            foreign_key_columns_by_referenced_table(columns)?;

        let on_delete_strategies_of_this_table =
            referencing_side_entry_points(context, &foreign_key_columns_by_referenced_table);

        contributions.merge(pairing::for_table(
            context,
            &foreign_key_columns_by_referenced_table,
        ));

        contributions.merge(TableContributions {
            struct_doc_comment: relationship_doc::of_struct(
                &context.singular_table_name,
                columns,
                &context.spacetimedsl_table.referencing_tables,
            ),
            ..TableContributions::default()
        });

        let methods = SpacetimeDSLTableMethods {
            create,
            get_all,
            get_count,
            on_delete_strategies_of_referencing_tables,
            on_delete_strategies_of_this_table,
            multi_column_indices: multi_column_index_methods(context),
            wrapper_methods: wrapper_methods(context, &foreign_key_columns_by_referenced_table),
        };

        Ok((methods, contributions))
    }
}

/// `create_<table>`, `get_all_<tables>` and `count_of_all_<tables>`.
fn table_level_methods(
    context: &MethodGenerationContext,
    contributions: &mut TableContributions,
) -> (
    Option<SpacetimeDSLMethod>,
    Option<SpacetimeDSLMethod>,
    Option<SpacetimeDSLMethod>,
) {
    let spacetimedsl_table = context.spacetimedsl_table;
    let is_singleton = spacetimedsl_table.is_singleton();

    // A table with a default has no create method, and skipping the generator is also what
    // withholds the `Create<Table>` argument struct: the generator is its only source.
    let create = match spacetimedsl_table.singleton_has_default() {
        true => None,
        false => {
            let (create, create_contributions) = for_create(context);
            contributions.merge(create_contributions);

            Some(create)
        }
    };

    // A singleton holds one row, so iterating and counting have nothing to say.
    let get_all = (!is_singleton).then(|| for_get_all(context));
    let get_count = (!is_singleton).then(|| for_get_count(context));

    (create, get_all, get_count)
}

/// The one-row and the many-row entry point for each kind of removal in `removal_kinds`, as
/// `(on_deletion, on_soft_deletion)`. `build` generates one entry point.
fn entry_points_per_removal(
    removal_kinds: impl IntoIterator<Item = Removal>,
    mut build: impl FnMut(Removal, &OneOrMultiple) -> SpacetimeDSLMethod,
) -> (Option<CascadeEntryPoints>, Option<CascadeEntryPoints>) {
    let mut on_deletion = None;
    let mut on_soft_deletion = None;

    for removal in removal_kinds {
        let entry_points = Some(CascadeEntryPoints {
            after_one_row: build(removal, &OneOrMultiple::One),
            after_multiple_rows: build(removal, &OneOrMultiple::Multiple),
        });

        match removal {
            Removal::Hard => on_deletion = entry_points,
            Removal::Soft => on_soft_deletion = entry_points,
        }
    }

    (on_deletion, on_soft_deletion)
}

/// The entry points a referenced table offers its referencing tables: one pair per kind of
/// removal it can perform, because the cascade a referencing table runs depends on which of
/// the two reached it.
fn referenced_side_entry_points(
    context: &MethodGenerationContext,
) -> Option<OnDeleteStrategiesOfReferencingTables> {
    let MethodGenerationContext {
        spacetimedb_table,
        spacetimedsl_table,
        primary_key_column,
        ..
    } = context;

    if spacetimedsl_table.referencing_tables.is_empty() {
        return None;
    }

    let removal_kinds = Removal::ALL
        .into_iter()
        .filter(|removal| removal.is_performed_by(spacetimedsl_table));

    let (on_deletion, on_soft_deletion) =
        entry_points_per_removal(removal_kinds, |removal, one_or_multiple| {
            for_referenced_by(
                removal,
                one_or_multiple,
                spacetimedb_table,
                spacetimedsl_table,
                primary_key_column,
            )
        });

    // A table which neither deletes nor soft-deletes rows runs no cascade, so it offers no
    // entry point, even though other tables reference it.
    (on_deletion.is_some() || on_soft_deletion.is_some()).then_some(
        OnDeleteStrategiesOfReferencingTables {
            on_deletion,
            on_soft_deletion,
        },
    )
}

/// The columns with a foreign key, grouped by the table the foreign key names.
///
/// The columns of a group share the cascade functions and the pairing checks of their
/// referenced table, so they have to agree on the type of its primary key and on its path.
/// That is checked for every group, whether or not its foreign keys declare a strategy.
fn foreign_key_columns_by_referenced_table(
    columns: &[Column],
) -> syn::Result<BTreeMap<&syn::Ident, Vec<&Column>>> {
    let mut columns_by_referenced_table: BTreeMap<&syn::Ident, Vec<&Column>> = BTreeMap::new();

    for column in columns {
        let Some(foreign_key) = &column.spacetimedsl_column.foreign_key else {
            continue;
        };

        let columns_of_the_referenced_table = columns_by_referenced_table
            .entry(&foreign_key.table_name)
            .or_default();

        if let Some(first_column) = columns_of_the_referenced_table.first() {
            reject_foreign_key_disagreeing_with(first_column, column)?;
        }

        columns_of_the_referenced_table.push(column);
    }

    Ok(columns_by_referenced_table)
}

/// Rejects `column` when its type, or the path of its referenced table, differs from
/// `first_column`'s, which references the same table.
fn reject_foreign_key_disagreeing_with(first_column: &Column, column: &Column) -> syn::Result<()> {
    // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 If Option is supported, the type of the primary key values needs to be without option and it's allowed to have both, option and non-option columns.
    if canonical_type(&column.rust_field.type_name_or_path)
        != canonical_type(&first_column.rust_field.type_name_or_path)
    {
        return Err(error::foreign_key_columns_type_mismatch(
            &column.rust_field.name,
        ));
    }

    if canonical_path(&foreign_key_of(column).path)
        != canonical_path(&foreign_key_of(first_column).path)
    {
        return Err(error::foreign_key_columns_path_mismatch(
            &column.rust_field.name,
        ));
    }

    Ok(())
}

/// A module path in one spelling per module, so `::other_crate::tables` and
/// `other_crate::tables` compare equal: the segments without a leading `::`.
fn canonical_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// The entry points this table offers each table it references: one pair per kind of
/// removal its foreign keys to that table declare a strategy for. A key that sets only
/// `on_soft_delete` contributes nothing to the deletion pair, and the other way round.
fn referencing_side_entry_points(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&syn::Ident, Vec<&Column>>,
) -> Vec<OnDeleteStrategiesOfTheReferencedTable> {
    let referencing_tables = match context.spacetimedsl_table.referencing_tables.is_empty() {
        true => ReferencingTables::Absent,
        false => ReferencingTables::Present,
    };

    foreign_key_columns_by_referenced_table
        .iter()
        .filter_map(|(referenced_table_name, columns_with_foreign_key)| {
            let removal_kinds = Removal::ALL.into_iter().filter(|removal| {
                columns_with_foreign_key.iter().any(|column| {
                    removal
                        .strategy_declared_by(foreign_key_of(column))
                        .is_some()
                })
            });

            let (on_deletion, on_soft_deletion) =
                entry_points_per_removal(removal_kinds, |removal, one_or_multiple| {
                    for_foreign_key(
                        removal,
                        one_or_multiple,
                        referencing_tables,
                        context,
                        referenced_table_name,
                        columns_with_foreign_key,
                    )
                });

            (on_deletion.is_some() || on_soft_deletion.is_some()).then_some(
                OnDeleteStrategiesOfTheReferencedTable {
                    on_deletion,
                    on_soft_deletion,
                },
            )
        })
        .collect()
}

/// The lookup methods each foreign key adds to its wrapper type.
fn wrapper_methods(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&syn::Ident, Vec<&Column>>,
) -> Vec<WrapperMethod> {
    foreign_key_columns_by_referenced_table
        .iter()
        .flat_map(|(referenced_table_name, columns_with_foreign_key)| {
            for_wrapper_methods(referenced_table_name, columns_with_foreign_key, context)
        })
        .collect()
}

/// The methods of every multi-column index.
fn multi_column_index_methods(context: &MethodGenerationContext) -> Vec<SpacetimeDSLColumnMethods> {
    context
        .spacetimedb_table
        .multi_column_indices
        .iter()
        .map(|multi_column_index| column_methods_for(multi_column_index, context))
        .collect()
}
