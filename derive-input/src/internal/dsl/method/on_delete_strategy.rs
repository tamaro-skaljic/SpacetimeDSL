//! What one on-delete strategy does to the rows that reference a deleted row.
//!
//! `Error` refuses, `Delete` removes them — and cascades further if they are themselves
//! referenced — `SetZero` clears the column, `Ignore` does nothing. Each is generated into
//! one arm of the match in [`super::foreign_key`].

use super::{
    context::{self},
    hook_call::hook_use_and_call,
    naming::referenced_table_function_name,
};
use crate::{
    api::{
        Column,
        dsl::{foreign_key::OnDeleteStrategy, table::SpacetimeDSLTable},
        runtime,
    },
    internal::{
        column::InternalColumn,
        dsl::{one_or_multiple::OneOrMultiple, singleton},
    },
};
use proc_macro2::TokenStream;
use quote::{TokenStreamExt, quote};
use syn::Ident;

/// How the generated code binds the row it iterates over or matches on.
#[derive(Clone, Copy)]
pub(in crate::internal) enum RowBinding {
    Immutable,
    Mutable,
}

/// Whether the index a strategy looks rows up through yields at most one row or many.
#[derive(Clone, Copy)]
pub(in crate::internal) enum IndexUniqueness {
    Unique,
    NonUnique,
}

/// Whether the table generating a cascade is itself referenced by another table, which
/// decides whether its strategies have to cascade further.
#[derive(Clone, Copy)]
pub(in crate::internal) enum ReferencingTables {
    Present,
    Absent,
}

pub(in crate::internal) fn on_delete_strategy_implementation(
    spacetimedsl_table: &SpacetimeDSLTable,
    referencing_tables: ReferencingTables,
    singular_table_name: &Ident,
    on_delete_strategy: &OnDeleteStrategy,
    columns_by_on_delete_strategy: Vec<&Column>,
    one_or_multiple: &OneOrMultiple,
    primary_key_column: &InternalColumn,
) -> TokenStream {
    let spacetimedb_call_prefix = quote! {
        dsl
            .db()
            .#singular_table_name()
    };

    let primary_key_column_name = &primary_key_column.rust_field_name;

    let singular_table_name_as_string = singular_table_name.to_string();

    // Deliberate empty slot, symmetric with strategy_after_all.
    let strategy_before_all = quote! {};
    let mut strategy_for_before_hook = TokenStream::default();
    let mut strategy_for_after_hook = TokenStream::default();

    let mut strategy_for_referenced_by = TokenStream::default();

    let mut strategy_by_column = vec![];
    let mut strategy_after_all = TokenStream::default();

    let is_singleton = spacetimedsl_table.is_singleton();

    for column in &columns_by_on_delete_strategy {
        let column_name = &column.rust_field.name;
        let column_name_as_string = column_name.to_string();

        // A singleton has no index on a foreign key column, so find its one row by the
        // injected primary key and check the column afterwards.
        let index_uniqueness = if is_singleton {
            // Singleton has at most 1 row, treat as unique
            IndexUniqueness::Unique
        } else {
            match column
                .spacetimedb_column
                .single_column_index
                .as_ref()
                .expect("A foreign key column always has a single-column index, except in singleton-tables")
                .is_unique
            {
                true => IndexUniqueness::Unique,
                false => IndexUniqueness::NonUnique,
            }
        };

        let row_finder = if is_singleton {
            // For singletons, find the single row by PK and check FK column manually
            let primary_key = singleton::primary_key_ident();
            let primary_key_value = singleton::primary_key_value();

            quote! {
                #spacetimedb_call_prefix.#primary_key().find(&#primary_key_value).filter(|row| row.#column_name == *primary_key_value_of_a_row_of_another_table_to_delete)
            }
        } else {
            match index_uniqueness {
                IndexUniqueness::Unique => {
                    quote! {
                        #spacetimedb_call_prefix.#column_name().find(primary_key_value_of_a_row_of_another_table_to_delete)
                    }
                }
                IndexUniqueness::NonUnique => {
                    quote! {
                        #spacetimedb_call_prefix.#column_name().filter(primary_key_value_of_a_row_of_another_table_to_delete)
                    }
                }
            }
        };

        let row_value_format = if is_singleton {
            // The injected primary key has no wrapper type to render it.
            let rendered_primary_key_value = singleton::rendered_primary_key_value();

            quote! { #rendered_primary_key_value.to_string() }
        } else {
            let wrapper_type_struct_name_or_path =
                context::primary_key_wrapper_type(primary_key_column);
            quote! { format!("{}", #wrapper_type_struct_name_or_path::new(#primary_key_column_name.clone())) }
        };

        let create_entry = runtime::deletion_result_entry(
            &singular_table_name_as_string,
            &column_name_as_string,
            on_delete_strategy,
            &row_value_format,
            &quote! { child_entries, },
        );

        let create_entry_and_add_it_to_entries = match one_or_multiple {
            OneOrMultiple::One => {
                quote! {
                    entries.push(
                        #create_entry
                    );
                }
            }
            OneOrMultiple::Multiple => {
                quote! {
                    entries.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).expect(&format!("{primary_key_value_of_a_row_of_another_table_to_delete} should exist in entries.")).push(#create_entry);
                }
            }
        };

        match on_delete_strategy {
            OnDeleteStrategy::Error => {
                strategy_by_column.push(strategy_by_row(
                    RowBinding::Immutable,
                    index_uniqueness,
                    &row_finder,
                    quote! {
                        error = true;

                        let child_entries = vec![];
                        let #primary_key_column_name = &row.#primary_key_column_name;
                        #create_entry_and_add_it_to_entries
                    },
                ));
            }
            OnDeleteStrategy::Delete => {
                // These two imports have to escape the per-row loop their guard sits in,
                // so they are hoisted into strategy_for_before_hook / _after_hook instead
                // of being emitted next to the call.
                let (use_before_delete_hook_trait, before_delete_hook) = hook_use_and_call(
                    &spacetimedsl_table.hooks.before_delete,
                    |hook_function_name| {
                        let hook_call = runtime::dsl_method_hooks_call(
                            hook_function_name,
                            &quote! { &dsl, &row },
                        );

                        quote! {
                            if let Err(error_raised_by_the_hook) = #hook_call {
                                error = true;
                                error_from_hook = Some(Box::new(error_raised_by_the_hook));
                                break 'outer;
                            }
                        }
                    },
                );
                strategy_for_before_hook = use_before_delete_hook_trait;

                let (use_after_delete_hook_trait, after_delete_hook) = hook_use_and_call(
                    &spacetimedsl_table.hooks.after_delete,
                    |hook_function_name| {
                        let hook_call = runtime::dsl_method_hooks_call(
                            hook_function_name,
                            &quote! { &dsl, &row },
                        );

                        quote! {
                            if let Err(error_raised_by_the_hook) = #hook_call {
                                error = true;
                                error_from_hook = Some(Box::new(error_raised_by_the_hook));
                                break 'outer;
                            }
                        }
                    },
                );
                strategy_for_after_hook = use_after_delete_hook_trait;

                match referencing_tables {
                    ReferencingTables::Absent => strategy_by_column.push(strategy_by_row(
                        RowBinding::Immutable,
                        index_uniqueness,
                        &row_finder,
                        quote! {
                            let child_entries = vec![];
                            let #primary_key_column_name = &row.#primary_key_column_name;
                            #create_entry_and_add_it_to_entries

                            #before_delete_hook

                            #spacetimedb_call_prefix
                                .#primary_key_column_name()
                                .delete(row.#primary_key_column_name);

                            #after_delete_hook
                        },
                    )),
                    ReferencingTables::Present => {
                        let format_str = format!(
                            "{primary_key_column_name} should exist in child_entries_by_primary_key_value_of_row_to_delete."
                        );
                        let create_entries_and_add_them_to_entries = quote! {
                            for (primary_key_value_of_a_row_of_another_table_to_delete, primary_key_values_of_rows_to_delete) in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete {
                                for #primary_key_column_name in &primary_key_values_of_rows_to_delete {
                                    let child_entries = child_entries_by_primary_key_value_of_row_to_delete.remove(&#primary_key_column_name).expect(&#format_str);
                                    #create_entry_and_add_it_to_entries
                                }
                            }
                        };

                        let failure = runtime::on_delete_strategy_failure(
                            &quote! { entries },
                            &quote! { error_from_hook },
                        );

                        let on_error_handler = quote! {
                            #create_entries_and_add_them_to_entries
                            return Err(#failure);
                        };

                        let error_strategy =
                            referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::Error,
                                &on_error_handler,
                            );

                        let delete_strategy =
                            referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::Delete,
                                &on_error_handler,
                            );

                        /*
                        let set_none_strategy =
                            referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                &singular_table_name_as_string,
                                OnDeleteStrategy::SetNone,
                                &on_error_handler
                            );
                        */

                        let set_zero_strategy =
                            referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::SetZero,
                                &on_error_handler,
                            );

                        let ignore_strategy =
                            referenced_table_function_call_for_strategy_implementation(
                                singular_table_name,
                                OnDeleteStrategy::Ignore,
                                &on_error_handler,
                            );

                        strategy_for_referenced_by = quote! {
                            let mut child_entries_by_primary_key_value_of_row_to_delete = std::collections::HashMap::new();
                            let mut row_to_delete_by_primary_key_value = std::collections::HashMap::new();
                            let mut primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete = std::collections::HashMap::new();
                        };

                        match one_or_multiple {
                            OneOrMultiple::One => strategy_for_referenced_by.append_all(quote! {
                                primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
                            }),
                            OneOrMultiple::Multiple => strategy_for_referenced_by.append_all(quote! {
                                for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                                    primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.insert(primary_key_value_of_a_row_of_another_table_to_delete, vec![]);
                                }
                            }),
                        };

                        let strategy_for_each_row = quote! {
                            if !child_entries_by_primary_key_value_of_row_to_delete.contains_key(&row.#primary_key_column_name) {
                                primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.get_mut(primary_key_value_of_a_row_of_another_table_to_delete).expect(&format!("{primary_key_value_of_a_row_of_another_table_to_delete} should exist in primary_key_values_of_rows_to_delete_by_primary_key_value_of_a_row_of_another_table_to_delete.")).push(row.#primary_key_column_name);
                                child_entries_by_primary_key_value_of_row_to_delete.insert(row.#primary_key_column_name, vec![]);
                            row_to_delete_by_primary_key_value.insert(row.#primary_key_column_name, row);
                            }
                        };

                        let delete_many_impl = quote! {
                            for #primary_key_column_name in &primary_key_values_of_rows_to_delete {
                                let row = row_to_delete_by_primary_key_value
                                    .get(#primary_key_column_name)
                                    .expect("Should exist");

                                #before_delete_hook

                                if !#spacetimedb_call_prefix
                                    .#primary_key_column_name()
                                    .delete(#primary_key_column_name) {
                                        #on_error_handler
                                    }

                                #after_delete_hook
                            }
                        };

                        strategy_after_all = quote! {
                            let primary_key_values_of_rows_to_delete = child_entries_by_primary_key_value_of_row_to_delete.keys().cloned().collect_vec();

                            #error_strategy

                            #delete_many_impl

                            #delete_strategy

                            //TODO https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 #set_none_strategy

                            #set_zero_strategy

                            #ignore_strategy

                            #create_entries_and_add_them_to_entries
                        };

                        strategy_by_column.push(strategy_by_row(
                            RowBinding::Immutable,
                            index_uniqueness,
                            &row_finder,
                            strategy_for_each_row,
                        ));
                    }
                };
            }
            OnDeleteStrategy::SetZero => {
                strategy_by_column.push(strategy_by_row(
                    RowBinding::Mutable,
                    index_uniqueness,
                    &row_finder,
                    quote! {
                        row.#column_name = 0;

                        let child_entries = vec![];
                        let #primary_key_column_name = &row.#primary_key_column_name;
                        #create_entry_and_add_it_to_entries

                        // FIXME: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/60 try_update instead of update and on error return Err(crate::spacetimedsl::error::SpacetimeDSLError);
                        #spacetimedb_call_prefix.#primary_key_column_name().update(row);
                    },
                ));
            }
            OnDeleteStrategy::Ignore => {
                strategy_by_column.push(strategy_by_row(
                    RowBinding::Immutable,
                    index_uniqueness,
                    &row_finder,
                    quote! {
                        let child_entries = vec![];
                        let #primary_key_column_name = &row.#primary_key_column_name;
                        #create_entry_and_add_it_to_entries
                    },
                ));
            }
        };
    }

    match one_or_multiple {
        OneOrMultiple::One => quote! {
            #strategy_before_all
            #strategy_for_before_hook
            #strategy_for_after_hook
            #strategy_for_referenced_by

            #(#strategy_by_column)*

            #strategy_after_all
        },
        OneOrMultiple::Multiple => quote! {
            #strategy_before_all
            #strategy_for_before_hook
            #strategy_for_after_hook
            #strategy_for_referenced_by

            for primary_key_value_of_a_row_of_another_table_to_delete in primary_key_values_of_rows_of_another_table_to_delete {
                #(#strategy_by_column)*
            }

            #strategy_after_all
        },
    }
}

fn strategy_by_row(
    row_binding: RowBinding,
    index_uniqueness: IndexUniqueness,
    row_finder: &TokenStream,
    strategy_for_each_row: TokenStream,
) -> TokenStream {
    let row_or_mut_row = match row_binding {
        RowBinding::Mutable => quote! {
            mut row
        },
        RowBinding::Immutable => quote! {
            row
        },
    };

    match index_uniqueness {
        IndexUniqueness::Unique => quote! {
            match #row_finder {
                None => {}
                Some(#row_or_mut_row) => {
                    #strategy_for_each_row
                }
            };
        },
        IndexUniqueness::NonUnique => quote! {
            for #row_or_mut_row in #row_finder {
                #strategy_for_each_row
            }
        },
    }
}

fn referenced_table_function_call_for_strategy_implementation(
    singular_table_name: &Ident,
    on_delete_strategy: OnDeleteStrategy,
    on_error_handler: &TokenStream,
) -> TokenStream {
    let referenced_table_function_name =
        referenced_table_function_name(&OneOrMultiple::Multiple, singular_table_name);
    let referenced_table_call = runtime::dsl_internals_call(
        &referenced_table_function_name,
        &quote! { dsl, #on_delete_strategy, &primary_key_values_of_rows_to_delete[..] },
    );

    quote! {
        match #referenced_table_call {
            Err(failure) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                    child_entries_by_primary_key_value_of_row_to_delete.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in child_entries_by_primary_key_value_of_row_to_delete.")).append(&mut child_entries);
                }

                if error_from_hook.is_none() {
                    error_from_hook = failure.error_from_hook;
                }

                #on_error_handler
            },
            Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                for (primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                    child_entries_by_primary_key_value_of_row_to_delete.get_mut(primary_key_value_of_a_row_to_delete).expect(&format!("{primary_key_value_of_a_row_to_delete} should exist in child_entries_by_primary_key_value_of_row_to_delete.")).append(&mut child_entries);
                }
            }
        };
    }
}
