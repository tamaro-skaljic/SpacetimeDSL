use crate::api::dsl::table::SpacetimeDSLTable;
use crate::api::{
    db::column::SpacetimeDBColumn,
    dsl::{
        auto_gen::UUIDVersion, column::SpacetimeDSLColumn, foreign_key::ForeignKey, getter::Getter,
        mut_getter::MutGetter, setter::Setter, wrapper::WrapperType,
    },
    rust::{column::RustField, table::RustStruct},
};
use crate::internal::column::ColumnTypeKind;
use crate::internal::dsl::error;
use spacetime_bindings_macro_input::sats::SatsField;

impl SpacetimeDSLColumn {
    pub(in crate::internal) fn try_parse(
        spacetimedsl_table: &SpacetimeDSLTable,
        field: &SatsField<'_>,
        rust_struct: &RustStruct,
        rust_field: &RustField,
        spacetimedb_column: &SpacetimeDBColumn,
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
            &spacetimedsl_table.has_delete_method,
            spacetimedsl_table.is_soft_deletable(),
            is_singleton,
            field,
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

        // Singleton PK column (id: u8) doesn't need getter/setter/mut_getter
        let is_singleton_pk = is_singleton && spacetimedb_column.is_primary_key;

        let (getter, mut_getter, setter) = if is_singleton_pk {
            (None, None, None)
        } else {
            (
                Some(Getter::map(rust_field, is_option, &wrapper_type)),
                MutGetter::map(rust_field, &wrapper_type),
                Setter::map(rust_field, is_option, &wrapper_type),
            )
        };

        Ok(SpacetimeDSLColumn {
            is_option,
            wrapper_type,
            foreign_key,
            auto_generated_uuid_version,
            getter,
            mut_getter,
            setter,
        })
    }
}
