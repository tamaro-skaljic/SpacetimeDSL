use std::collections::BTreeSet;

use crate::api::db::{index::IndexType, table::SpacetimeDBTable};
use crate::api::dsl::reference::ReferencingTable;
use crate::api::dsl::table::{SingletonKind, SpacetimeDSLTable};
use crate::internal::DSLData;
use crate::internal::dsl::hook::DeclaredHooks;
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::table::ColumnArgs;

#[derive(Clone, Copy)]
enum TimestampRole {
    CreatedAt,
    UpdatedAt,
}

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
            dsl_data.singleton,
            DeclaredHooks {
                before_insert: dsl_data.before_insert_hook,
                before_update: dsl_data.before_update_hook,
                before_delete: dsl_data.before_delete_hook,
                after_insert: dsl_data.after_insert_hook,
                after_update: dsl_data.after_update_hook,
                after_delete: dsl_data.after_delete_hook,
            },
        );

        let has_update_method = &dsl_data.update_method;
        let has_delete_method = &dsl_data.delete_method;
        let mut all_columns_are_private = true;

        let has_update_method = match has_update_method {
            None => {
                for field in &column_args.fields {
                    if matches!(
                        field.vis,
                        syn::Visibility::Public(_) | syn::Visibility::Restricted(_)
                    ) {
                        return Err(syn::Error::new_spanned(
                        &column_args.original_struct_name,
                        "HasUpdateMethod must be set in `#[dsl(method(update = HasUpdateMethod))]`\nBecause you have at least one column which is not private, you should set `#[dsl(method(update = true))]`.\nIf, instead, you want immutable rows in this table which don't have setters and can't be updated, all columns must be private and you must specify `#[dsl(method(update = false))]`.".to_string(),
                    ));
                    }
                }
                return Err(syn::Error::new_spanned(
                    &column_args.original_struct_name,
                    "HasUpdateMethod must be set in `#[dsl(method(update = HasUpdateMethod))]`, e.g. `update = false`.\nBecause all your columns are private, you should set `#[dsl(method(update = false))]`.\nIf, instead, you want mutable rows in this table which have setters and can be updated, at least one column must be non-private or named `modified_at`/`updated_at` and you must specify `#[dsl(method(update = true))]`.",
                ));
            }
            Some(has_update_method) => *has_update_method,
        };

        let soft_delete_marker = super::soft_delete::try_parse(
            dsl_data.soft_delete_method,
            dsl_data.singleton,
            column_args,
            &column_args.original_struct_name,
        )?;

        let mut on_insert_set_current_timestamp_column_name = None;
        let mut on_update_set_current_timestamp_column_name = None;

        let mut referencing_tables = vec![];

        for field in &column_args.fields {
            let refs = ReferencingTable::try_parse(&has_delete_method.unwrap_or(true), field)?;
            if referencing_tables.is_empty() {
                referencing_tables = refs;
            }

            if matches!(
                field.vis,
                syn::Visibility::Public(_) | syn::Visibility::Restricted(_)
            ) {
                if !has_update_method {
                    return Err(syn::Error::new_spanned(
                        field.vis,
                        format!(
                            "All columns in a table with disabled `update` DSL method should be private! Found: {:?}",
                            field.vis.to_token_stream().to_string()
                        ),
                    ));
                }
                all_columns_are_private = false;
            }

            let column_name = field.name.as_ref().expect("should have a name");
            let timestamp_role = get_timestamp_role(field)?;
            let field_type = field.ty.to_token_stream().to_string();

            if dsl_data.singleton == Some(SingletonKind::WithDefault)
                && is_bare_timestamp_type(&field_type)
            {
                return Err(syn::Error::new_spanned(
                    field.ty,
                    format!(
                        "A column on a `singleton(with_default)` table should have the type `Option<spacetimedb::Timestamp>`! Found: {field_type}"
                    ),
                ));
            }

            if matches!(timestamp_role, Some(TimestampRole::CreatedAt)) {
                if on_insert_set_current_timestamp_column_name.is_some() {
                    return Err(syn::Error::new_spanned(
                        field.ident.expect("a named field has an identifier"),
                        "Multiple columns claim the `created_at` role! Only one column is allowed.",
                    ));
                };
                let created_at_type_is_valid = match dsl_data.singleton {
                    Some(SingletonKind::WithDefault) => is_optional_timestamp_type(&field_type),
                    _ => is_bare_timestamp_type(&field_type),
                };
                if !created_at_type_is_valid {
                    return Err(syn::Error::new_spanned(
                        field.ty,
                        format!(
                            "A column with the `created_at` role should have the type `{}`! Found: {field_type}",
                            match dsl_data.singleton {
                                Some(SingletonKind::WithDefault) =>
                                    "Option<spacetimedb::Timestamp>",
                                _ => "spacetimedb::Timestamp",
                            }
                        ),
                    ));
                }

                match field.vis {
                    syn::Visibility::Public(_) => {
                        return Err(syn::Error::new_spanned(
                            field.vis,
                            "A column with the `created_at` role should have `Visibility::Inherited`! Found: Visibility::Public",
                        ));
                    }
                    syn::Visibility::Restricted(_) => {
                        return Err(syn::Error::new_spanned(
                            field.vis,
                            "A column with the `created_at` role should have `Visibility::Inherited`! Found: Visibility::Restricted",
                        ));
                    }
                    syn::Visibility::Inherited => {
                        on_insert_set_current_timestamp_column_name =
                            Some(format_ident!("{column_name}"));
                    }
                }
            }
            if matches!(timestamp_role, Some(TimestampRole::UpdatedAt)) {
                if on_update_set_current_timestamp_column_name.is_some() {
                    return Err(syn::Error::new_spanned(
                        field.ident.expect("a named field has an identifier"),
                        "Multiple columns claim the `updated_at` role! Only one column is allowed.",
                    ));
                };

                if !has_update_method {
                    return Err(syn::Error::new_spanned(
                        field.ident.expect("a named field has an identifier"),
                        "A column with the `updated_at` role requires the `update` method to be enabled in `#[dsl(method(update = true))]`!",
                    ));
                }

                if !field_type.eq("Timestamp")
                    && !field_type.eq("spacetimedb :: Timestamp")
                    && !field_type.eq("Option < Timestamp >")
                    && !field_type.eq("Option < spacetimedb :: Timestamp >")
                {
                    return Err(syn::Error::new_spanned(
                        field.ty,
                        format!(
                            "A column with the `updated_at` role should have the type `spacetimedb::Timestamp` or `Option<spacetimedb::Timestamp>`! Found: {field_type}"
                        ),
                    ));
                }

                match field.vis {
                    syn::Visibility::Public(_) => {
                        return Err(syn::Error::new_spanned(
                            field.vis,
                            "A column with the `updated_at` role should have `Visibility::Inherited`! Found: Visibility::Public",
                        ));
                    }
                    syn::Visibility::Restricted(_) => {
                        return Err(syn::Error::new_spanned(
                            field.vis,
                            "A column with the `updated_at` role should have `Visibility::Inherited`! Found: Visibility::Restricted",
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
            && !has_update_method
            && on_update_set_current_timestamp_column_name.is_some()
        {
            return Err(syn::Error::new_spanned(
                &column_args.original_struct_name,
                "Because you have a column named `modified_at`/`updated_at`, you must specify `#[dsl(method(update = true))]`\nIf, instead, you want immutable rows in this table which don't have setters and can't be updated, all columns must be private, you must remove the `modified_at`/`updated_at` column and you must specify `#[dsl(method(update = false))]`.",
            ));
        }

        Ok((
            spacetimedb_table,
            SpacetimeDSLTable {
                singleton: dsl_data.singleton,
                plural_name: dsl_data.plural_name,
                has_update_method,
                has_delete_method: has_delete_method.unwrap_or(true),
                soft_delete_marker,
                on_insert_set_current_timestamp_column_name,
                on_update_set_current_timestamp_column_name,
                referencing_tables,
                compile_error_checks: BTreeSet::new(),
                // `TableContributions::apply_to` fills this in, after the create method is
                // generated.
                create_dsl_method_arg: None,
                hooks,
            },
        ))
    }
}

fn get_timestamp_role(
    field: &spacetime_bindings_macro_input::sats::SatsField<'_>,
) -> syn::Result<Option<TimestampRole>> {
    let column_name = field.name.as_ref().expect("should have a name");
    let has_created_at_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("created_at"));
    let has_updated_at_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("updated_at"));
    let is_created_at =
        column_name.eq("created_at") || column_name.eq("inserted_at") || has_created_at_attribute;
    let is_updated_at =
        column_name.eq("modified_at") || column_name.eq("updated_at") || has_updated_at_attribute;

    if is_created_at && is_updated_at {
        return Err(syn::Error::new_spanned(
            field.ident.expect("a named field has an identifier"),
            "A column cannot be both `created_at` and `updated_at`.",
        ));
    }

    Ok(match (is_created_at, is_updated_at) {
        (true, false) => Some(TimestampRole::CreatedAt),
        (false, true) => Some(TimestampRole::UpdatedAt),
        (false, false) => None,
        (true, true) => unreachable!(),
    })
}

fn is_bare_timestamp_type(field_type: &str) -> bool {
    field_type.eq("Timestamp") || field_type.eq("spacetimedb :: Timestamp")
}

fn is_optional_timestamp_type(field_type: &str) -> bool {
    field_type.eq("Option < Timestamp >") || field_type.eq("Option < spacetimedb :: Timestamp >")
}
