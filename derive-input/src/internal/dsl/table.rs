use std::collections::BTreeSet;

use crate::api::db::{index::IndexType, table::SpacetimeDBTable};
use crate::api::dsl::reference::ReferencingTable;
use crate::api::dsl::table::{SingletonKind, SpacetimeDSLTable};
use crate::internal::DSLData;
use crate::internal::dsl::error;
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
                before_soft_delete: dsl_data.before_soft_delete_hook,
                after_insert: dsl_data.after_insert_hook,
                after_update: dsl_data.after_update_hook,
                after_delete: dsl_data.after_delete_hook,
                after_soft_delete: dsl_data.after_soft_delete_hook,
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
                        return Err(error::missing_update_method_with_non_private_column(
                            &column_args.original_struct_name,
                        ));
                    }
                }
                return Err(error::missing_update_method_with_only_private_columns(
                    &column_args.original_struct_name,
                ));
            }
            Some(has_update_method) => *has_update_method,
        };

        let soft_delete_marker = super::soft_delete::try_parse(
            dsl_data.soft_delete_method.as_ref(),
            dsl_data.singleton,
            column_args,
        )?;

        let mut on_insert_set_current_timestamp_column_name = None;
        let mut on_update_set_current_timestamp_column_name = None;

        let mut referencing_tables = vec![];

        for field in &column_args.fields {
            let refs = ReferencingTable::try_parse(
                &has_delete_method.unwrap_or(true),
                soft_delete_marker.is_some(),
                field,
            )?;
            if referencing_tables.is_empty() {
                referencing_tables = refs;
            }

            if matches!(
                field.vis,
                syn::Visibility::Public(_) | syn::Visibility::Restricted(_)
            ) {
                if !has_update_method {
                    return Err(error::non_private_column_without_update_method(field.vis));
                }
                all_columns_are_private = false;
            }

            let column_name = field.name.as_ref().expect("should have a name");
            let timestamp_role = get_timestamp_role(field)?;
            let field_type = field.ty.to_token_stream().to_string();

            if dsl_data.singleton == Some(SingletonKind::WithDefault)
                && is_bare_timestamp_type(&field_type)
            {
                return Err(error::bare_timestamp_on_singleton_with_default(field.ty));
            }

            if matches!(timestamp_role, Some(TimestampRole::CreatedAt)) {
                if on_insert_set_current_timestamp_column_name.is_some() {
                    return Err(error::multiple_set_on_create_columns(
                        field.ident.expect("a named field has an identifier"),
                    ));
                };
                let set_on_create_type_is_valid = match dsl_data.singleton {
                    Some(SingletonKind::WithDefault) => is_optional_timestamp_type(&field_type),
                    _ => is_bare_timestamp_type(&field_type),
                };
                if !set_on_create_type_is_valid {
                    return Err(error::set_on_create_column_type_mismatch(
                        field.ty,
                        dsl_data.singleton,
                    ));
                }

                if !matches!(field.vis, syn::Visibility::Inherited) {
                    return Err(error::set_on_create_column_not_private(field.vis));
                }
                on_insert_set_current_timestamp_column_name = Some(format_ident!("{column_name}"));
            }
            if matches!(timestamp_role, Some(TimestampRole::UpdatedAt)) {
                if on_update_set_current_timestamp_column_name.is_some() {
                    return Err(error::multiple_set_on_update_columns(
                        field.ident.expect("a named field has an identifier"),
                    ));
                };

                if !has_update_method {
                    return Err(error::set_on_update_column_without_update_method(
                        field.ident.expect("a named field has an identifier"),
                    ));
                }

                if !field_type.eq("Timestamp")
                    && !field_type.eq("spacetimedb :: Timestamp")
                    && !field_type.eq("Option < Timestamp >")
                    && !field_type.eq("Option < spacetimedb :: Timestamp >")
                {
                    return Err(error::set_on_update_column_type_mismatch(field.ty));
                }

                if !matches!(field.vis, syn::Visibility::Inherited) {
                    return Err(error::set_on_update_column_not_private(field.vis));
                }
                on_update_set_current_timestamp_column_name = Some(format_ident!("{column_name}"));
            }
        }

        if all_columns_are_private
            && !has_update_method
            && on_update_set_current_timestamp_column_name.is_some()
        {
            return Err(error::update_method_disabled_with_set_on_update_column(
                &column_args.original_struct_name,
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
    let has_set_on_create_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("set_on_create"));
    let has_set_on_update_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("set_on_update"));
    let is_set_on_create = column_name.eq("created_at")
        || column_name.eq("inserted_at")
        || has_set_on_create_attribute;
    let is_set_on_update = column_name.eq("modified_at")
        || column_name.eq("updated_at")
        || has_set_on_update_attribute;

    if is_set_on_create && is_set_on_update {
        return Err(error::set_on_create_and_set_on_update(
            field.ident.expect("a named field has an identifier"),
        ));
    }

    Ok(match (is_set_on_create, is_set_on_update) {
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
