use {
    crate::{
        api::{
            db::column::SpacetimeDBColumn,
            dsl::{
                auto_gen::UUIDVersion, column::SpacetimeDSLColumn, foreign_key::ForeignKey,
                getter::Getter, mut_getter::MutGetter, setter::Setter, table::SpacetimeDSLTable,
                wrapper::WrapperType,
            },
            rust::{column::RustField, table::RustStruct},
        },
        internal::{column::ColumnTypeKind, error},
    },
    spacetime_bindings_macro_input::sats::SatsField,
    syn::Ident,
};

impl SpacetimeDSLColumn {
    pub(crate) fn try_parse(
        spacetimedsl_table: &SpacetimeDSLTable,
        field: &SatsField<'_>,
        rust_struct: &RustStruct,
        rust_field: &RustField,
        spacetimedb_column: &SpacetimeDBColumn,
        unique_multi_column_index: Option<&Ident>,
    ) -> syn::Result<SpacetimeDSLColumn> {
        let is_singleton = spacetimedsl_table.is_singleton();
        let is_option =
            ColumnTypeKind::of(&rust_field.type_name_or_path) == ColumnTypeKind::Optional;

        let wrapper_type = WrapperType::try_parse(rust_struct, rust_field, field)?;

        let auto_generated_uuid_version = UUIDVersion::try_parse(
            field,
            rust_field,
            &wrapper_type,
            spacetimedsl_table.singleton_has_default(),
        )?;

        if !is_singleton && spacetimedb_column.is_primary_key && wrapper_type.is_none() {
            return Err(error::primary_key_without_wrapper(&rust_field.name));
        }

        let foreign_key = ForeignKey::try_parse(
            spacetimedsl_table.has_delete_method,
            spacetimedsl_table.is_soft_deletable(),
            is_singleton,
            field,
            spacetimedb_column,
            ColumnTypeKind::of(&rust_field.type_name_or_path),
            unique_multi_column_index,
        )?;

        if foreign_key.is_some() {
            match &wrapper_type {
                Some(wrapper_type) => match wrapper_type {
                    WrapperType::Created(_) => {
                        return Err(error::foreign_key_with_created_wrapper(&rust_field.name));
                    }
                    WrapperType::Used(_) => {}
                },
                None => {
                    return Err(error::foreign_key_without_wrapper(&rust_field.name));
                }
            }
        }

        // Through `super`, because importing the module would import the attribute's symbol,
        // which has the same name, and turn this binding into a pattern matching it.
        let creation_default = super::creation_default::try_parse(
            field,
            &rust_field.name,
            spacetimedb_column,
            spacetimedsl_table,
            auto_generated_uuid_version,
        )?;

        // Singleton PK column (id: u8) doesn't need getter/setter/mut_getter
        let is_singleton_pk = is_singleton && spacetimedb_column.is_primary_key;

        let (getter, mut_getter, setter) = if is_singleton_pk {
            (None, None, None)
        } else {
            (
                Some(Getter::map(
                    rust_field,
                    is_option,
                    &wrapper_type,
                    foreign_key.as_ref(),
                )),
                MutGetter::map(rust_field, &wrapper_type),
                Setter::map(rust_field, is_option, &wrapper_type, foreign_key.as_ref()),
            )
        };

        Ok(SpacetimeDSLColumn {
            is_option,
            wrapper_type,
            foreign_key,
            auto_generated_uuid_version,
            creation_default,
            getter,
            mut_getter,
            setter,
        })
    }
}

/// A primary key named `<table>_<name>` repeats the table's name in every method generated
/// for it, such as `get_entity_by_entity_id`.
pub fn reject_primary_key_prefixed_with_table_name(
    column_name: &Ident,
    is_primary_key: bool,
    singular_table_name: &Ident,
) -> syn::Result<()> {
    if is_primary_key
        && column_name
            .to_string()
            .starts_with(&singular_table_name.to_string())
    {
        return Err(error::primary_key_prefixed_with_table_name(
            column_name,
            singular_table_name,
        ));
    }

    Ok(())
}
