use std::collections::HashSet;

use crate::api::db::{index::IndexType, table::SpacetimeDBTable};
use crate::api::dsl::reference::ReferencingTable;
use crate::api::dsl::table::SpacetimeDSLTable;
use crate::internal::DSLData;
use proc_macro2::Span;
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::table::ColumnArgs;

impl SpacetimeDSLTable {
    pub(in crate::internal) fn try_parse(
        dsl_data: DSLData,
        column_args: &ColumnArgs<'_>,
        mut spacetimedb_table: SpacetimeDBTable,
    ) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable)> {
        let unique_indices = dsl_data.unique_indices;

        for unique_index_name in unique_indices {
            for multi_column_index in &mut spacetimedb_table.multi_column_indices {
                if let IndexType::BTreeMultiColumn { columns: _ } = &multi_column_index.index_type
                    && multi_column_index.name.eq(&unique_index_name)
                {
                    multi_column_index.is_unique = true;
                }
            }
        }

        let hooks = super::hook::build(
            &spacetimedb_table.singular_name,
            dsl_data.before_insert_hook,
            dsl_data.before_update_hook,
            dsl_data.before_delete_hook,
            dsl_data.after_insert_hook,
            dsl_data.after_update_hook,
            dsl_data.after_delete_hook,
        );

        match column_args
            .primary_key_column
            .as_ref()
            .expect("The table should have a `#[primary_key]` column!")
            .vis
        {
            syn::Visibility::Public(_) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "A `#[primary_key]` column should have `Visibility::Inherited` (private)! Found: Visibility::Public",
                ));
            }
            syn::Visibility::Restricted(_) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "A `#[primary_key]` column should have `Visibility::Inherited` (private)! Found: Visibility::Restricted",
                ));
            }
            syn::Visibility::Inherited => {}
        }

        let has_update_method = &dsl_data.update_method;
        let has_delete_method = &dsl_data.delete_method;
        let mut all_columns_are_private = true;

        if has_update_method.is_none() {
            for field in &column_args.fields {
                if matches!(
                    field.vis,
                    syn::Visibility::Public(_) | syn::Visibility::Restricted(_)
                ) {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "HasUpdateMethod must be set in `#[dsl(method(update = HasUpdateMethod))]`\nBecause you have at least one column which is not private, you should set `#[dsl(method(update = true))]`.\nIf, instead, you want immutable rows in this table which don't have setters and can't be updated, all columns must be private and you must specify `#[dsl(method(update = false))]`.".to_string(),
                    ));
                }
            }
            return Err(syn::Error::new(
                Span::call_site(),
                "HasUpdateMethod must be set in `#[dsl(method(update = HasUpdateMethod))]`, e.g. `update = false`.\nBecause all your columns are private, you should set `#[dsl(method(update = false))]`.\nIf, instead, you want mutable rows in this table which have setters and can be updated, at least one column must be non-private or named `modified_at`/`updated_at` and you must specify `#[dsl(method(update = true))]`.",
            ));
        }

        let mut on_insert_set_current_timestamp_column_name = None;
        let mut on_update_set_current_timestamp_column_name = None;

        let mut referencing_tables = vec![];

        for field in &column_args.fields {
            let refs = ReferencingTable::try_parse(field)?;
            if referencing_tables.is_empty() {
                referencing_tables = refs;
            }

            if matches!(
                field.vis,
                syn::Visibility::Public(_) | syn::Visibility::Restricted(_)
            ) {
                if !has_update_method.unwrap() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!(
                            "All columns in a table with disabled `update` DSL method should be private! Found: {:?}",
                            field.vis.to_token_stream().to_string()
                        ),
                    ));
                }
                all_columns_are_private = false;
            }

            let column_name = field.name.as_ref().expect("should have a name");

            if column_name.eq("created_at") || column_name.eq("inserted_at") {
                if on_insert_set_current_timestamp_column_name.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Multiple columns for `on_insert_set_current_timestamp`: `created_at` and `inserted_at`! Only one column is allowed.".to_string(),
                    ));
                };
                let field_type = field.ty.to_token_stream().to_string();
                if !field_type.eq("Timestamp") && !field_type.eq("spacetimedb :: Timestamp") {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!(
                            "A column with name `created_at` or `inserted_at` should have the type `spacetimedb::Timestamp`! Found: {field_type}"
                        ),
                    ));
                }

                match field.vis {
                    syn::Visibility::Public(_) => {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "A column with name `created_at` or `inserted_at` should have `Visibility::Inherited`! Found: Visibility::Public",
                        ));
                    }
                    syn::Visibility::Restricted(_) => {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "A column with name `created_at` or `inserted_at` should have `Visibility::Inherited`! Found: Visibility::Restricted",
                        ));
                    }
                    syn::Visibility::Inherited => {
                        on_insert_set_current_timestamp_column_name =
                            Some(format_ident!("{column_name}"));
                    }
                }
            }
            if column_name.eq("modified_at") || column_name.eq("updated_at") {
                if on_update_set_current_timestamp_column_name.is_some() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "Multiple columns for `on_update_set_current_timestamp`: `modified_at` and `updated_at`! Only one column is allowed.".to_string(),
                    ));
                };

                if !has_update_method.unwrap() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        "A column with name `modified_at` or `updated_at` requires the `update` method to be enabled in `#[dsl(method(update = true))]`!".to_string(),
                    ));
                }

                let field_type = field.ty.to_token_stream().to_string();
                if !field_type.eq("Timestamp")
                    && !field_type.eq("spacetimedb :: Timestamp")
                    && !field_type.eq("Option < Timestamp >")
                    && !field_type.eq("Option < spacetimedb :: Timestamp >")
                {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!(
                            "A column with name `modified_at` or `updated_at` should have the type `spacetimedb::Timestamp` or `Option<spacetimedb::Timestamp>`! Found: {field_type}"
                        ),
                    ));
                }

                match field.vis {
                    syn::Visibility::Public(_) => {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "A column with name `modified_at` or `updated_at` should have `Visibility::Inherited`! Found: Visibility::Public",
                        ));
                    }
                    syn::Visibility::Restricted(_) => {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            "A column with name `modified_at` or `updated_at` should have `Visibility::Inherited`! Found: Visibility::Restricted",
                        ));
                    }
                    syn::Visibility::Inherited => {
                        on_update_set_current_timestamp_column_name =
                            Some(format_ident!("{column_name}"));
                    }
                }
            }
        }

        if all_columns_are_private
            && !has_update_method.unwrap()
            && on_update_set_current_timestamp_column_name.is_some()
        {
            return Err(syn::Error::new(
                Span::call_site(),
                "Because you have a column named `modified_at`/`updated_at`, you must specify `#[dsl(method(update = true))]\nIf, instead, you want immutable rows in this table which don't have setters and can't be updated, all columns must be private, you must remove the `modified_at`/`updated_at` column and you must specify `#[dsl(method(update = false))]`.",
            ));
        }

        Ok((
            spacetimedb_table,
            SpacetimeDSLTable {
                plural_name: dsl_data.plural_name,
                has_update_method: has_update_method.unwrap(),
                has_delete_method: has_delete_method.unwrap_or(true),
                on_insert_set_current_timestamp_column_name,
                on_update_set_current_timestamp_column_name,
                referencing_tables,
                compile_error_checks: HashSet::new(),
                // is set later in method.rs.
                create_dsl_method_arg: None,
                hooks,
            },
        ))
    }
}
