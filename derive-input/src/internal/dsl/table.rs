use {
    crate::{
        api::dsl::{
            reference::ReferencingTable,
            table::{SingletonKind, SpacetimeDSLTable, SpacetimeDSLTableKind},
        },
        internal::{DSLData, column::ColumnTypeKind, dsl::column_role, error},
    },
    quote::format_ident,
    spacetime_bindings_macro_input::table::ColumnArgs,
    std::collections::BTreeSet,
    syn::{Ident, Type},
};

#[derive(Clone, Copy)]
enum TimestampRole {
    CreatedAt,
    UpdatedAt,
}

impl SpacetimeDSLTableKind {
    /// The kind of the singleton, or `None` for an ordinary table.
    pub(crate) fn singleton(&self) -> Option<SingletonKind> {
        match self {
            SpacetimeDSLTableKind::Singleton(singleton_kind) => Some(*singleton_kind),
            SpacetimeDSLTableKind::Normal { .. } => None,
        }
    }
}

impl SpacetimeDSLTable {
    pub(crate) fn is_singleton(&self) -> bool {
        self.kind.singleton().is_some()
    }

    pub(crate) fn is_soft_deletable(&self) -> bool {
        self.soft_delete_marker.is_some()
    }

    /// Whether `get_<table>` falls back to `DefaultSingleton::get_default` instead of failing.
    pub(crate) fn singleton_has_default(&self) -> bool {
        self.kind.singleton() == Some(SingletonKind::WithDefault)
    }

    pub(crate) fn try_parse(
        dsl_data: DSLData,
        column_args: &ColumnArgs<'_>,
        singular_table_name: &Ident,
    ) -> syn::Result<SpacetimeDSLTable> {
        let singleton = dsl_data.kind.singleton();
        let hooks = super::hook::build(singular_table_name, singleton, &dsl_data.declared_hooks);

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
            singleton,
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
            let field_type = field.ty;

            if singleton == Some(SingletonKind::WithDefault) && is_bare_timestamp_type(field_type) {
                return Err(error::bare_timestamp_on_singleton_with_default(field.ty));
            }

            if matches!(timestamp_role, Some(TimestampRole::CreatedAt)) {
                if on_insert_set_current_timestamp_column_name.is_some() {
                    return Err(error::multiple_set_on_create_columns(
                        field.ident.expect("a named field has an identifier"),
                    ));
                };
                let set_on_create_type_is_valid = match singleton {
                    Some(SingletonKind::WithDefault) => is_optional_timestamp_type(field_type),
                    _ => is_bare_timestamp_type(field_type),
                };
                if !set_on_create_type_is_valid {
                    return Err(error::set_on_create_column_type_mismatch(
                        field.ty, singleton,
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

                if !is_bare_timestamp_type(field_type) && !is_optional_timestamp_type(field_type) {
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

        Ok(SpacetimeDSLTable {
            kind: dsl_data.kind,
            has_update_method,
            has_delete_method: has_delete_method.unwrap_or(true),
            soft_delete_marker,
            on_insert_set_current_timestamp_column_name,
            on_update_set_current_timestamp_column_name,
            referencing_tables,
            // `TableContributions::apply_to` fills these in, after the methods are generated.
            compile_error_checks: BTreeSet::new(),
            compile_error_check_imports: vec![],
            create_dsl_method_arg: None,
            hooks,
        })
    }
}

fn get_timestamp_role(
    field: &spacetime_bindings_macro_input::sats::SatsField<'_>,
) -> syn::Result<Option<TimestampRole>> {
    let column_name = field.name.as_ref().expect("should have a name");
    let has_set_on_create_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path() == super::set_on_create);
    let has_set_on_update_attribute = field
        .original_attrs
        .iter()
        .any(|attribute| attribute.path() == super::set_on_update);
    let is_set_on_create = has_set_on_create_attribute
        || column_role::claims(&column_role::SET_ON_CREATE_COLUMN_NAMES, column_name);
    let is_set_on_update = has_set_on_update_attribute
        || column_role::claims(&column_role::SET_ON_UPDATE_COLUMN_NAMES, column_name);

    Ok(match (is_set_on_create, is_set_on_update) {
        (true, true) => {
            return Err(error::set_on_create_and_set_on_update(
                field.ident.expect("a named field has an identifier"),
            ));
        }
        (true, false) => Some(TimestampRole::CreatedAt),
        (false, true) => Some(TimestampRole::UpdatedAt),
        (false, false) => None,
    })
}

fn is_bare_timestamp_type(field_type: &Type) -> bool {
    ColumnTypeKind::of_type(field_type) == ColumnTypeKind::Timestamp
}

fn is_optional_timestamp_type(field_type: &Type) -> bool {
    ColumnTypeKind::of_option_argument_of_type(field_type) == Some(ColumnTypeKind::Timestamp)
}
