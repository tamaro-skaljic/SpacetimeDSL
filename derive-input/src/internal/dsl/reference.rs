use super::referenced_by;
use crate::api::dsl::reference::ReferencingTable;
use crate::internal::dsl::{path, table};
use quote::ToTokens;
use spacetime_bindings_macro_input::match_meta;
use spacetime_bindings_macro_input::sats::SatsField;
use spacetime_bindings_macro_input::sym::primary_key;
use spacetime_bindings_macro_input::util::check_duplicate;
use syn::{Ident, Path};

/**
 * TODO: Reference
 * - If the table has at least one Reference
 *   - (the dsl reference is passed as argument to any function as well, though it isn't written below)
 *   - Generate a function (for delete_one) in the same module.
 *     - Name: perform_actions_after_{table_name}_deletion
 *     - Arg: #table_name: &#column_type
 *     - Return Type: Result<(), UniqueConstraintViolationError>
 *     - Impl:
 *       - For each referenced table:
 *         - #referenced_path.perform_{referenced_table_name}_actions_after_{table_name}_deletion(#table_name)?;
 *       - Ok(())
 *   - Generate another function (delete_many) in the same module.
 *     - Name: perform_actions_after_{table_name}_deletions
 *     - Arg: #plural_table_name: Vec<&#column_type>
 *     - Return Type: Result<(), UniqueConstraintViolationError>
 *     - Impl:
 *       - For each referenced table:
 *         - #referenced_path.perform_{referenced_table_name}_actions_after_{table_name}_deletions(#plural_table_name)?;
 *       - Ok(())
 *   - Use the function
 *     - in delete_one:
 *       - for_single_column_index:
 *         - Change `return [...].delete()` to `[...].delete()?;`
 *         - perform_actions_after_{table_name}_deletion(#column_value)?;
 *         - true
 *       - for_multi_column_index:
 *         - Change `return [...].delete()` to `[...].delete()?;`
 *         - perform_actions_after_{table_name}_deletion(#field_name_for_found_value.unwrap().#primary_key_column_name)?;
 *         - true
 *     - in delete_many:
 *       - for_single_column_index:
 *         - After `#into_option`
 *           - let #plural_table_name = self.ctx().db().#table_name().#column_name().filter(#column_value).map(|#table_name| => #table_name.#primary_key_column_name).collect();
 *         - Change `return [...].delete()` to `let count = [...].delete()?;`
 *         - perform_actions_after_{table_name}_deletions(#plural_table_name)?;
 *         - count
 *       - for_multi_column_index:
 *         - After `#into_option`
 *           - let #plural_table_name = self.ctx().db().#table_name().#index_name().filter((#(#column_values),*)).map(|#table_name| => #table_name.#primary_key_column_name).collect();
 *         - Change `return [...].delete()` to `let count = [...].delete()?;`
 *         - perform_actions_after_{table_name}_deletions(#plural_table_name)?;
 *         - count
 */
impl ReferencingTable {
    // TODO: There should be a proper error message if the column which references the primary_key column has not a valid type (This column: T | Option<T>, the other column: T). But this probably won't work from inside rust macros, more likely in a build.rs. Currently it's a compilation error.
    pub(in crate::internal) fn try_parse(
        field: &SatsField<'_>,
    ) -> syn::Result<Vec<ReferencingTable>> {
        let mut referencing_tables: Vec<ReferencingTable> = vec![];

        let mut is_primary_key = false;
        for attr in field.original_attrs {
            if attr.meta.path().eq(&primary_key) {
                is_primary_key = true;
                break;
            }
        }

        for attr in field.original_attrs {
            if attr.meta.path().ne(&referenced_by) {
                continue;
            }

            if !is_primary_key {
                return Err(syn::Error::new_spanned(
                    &attr,
                    "`#[referenced_by]` is only allowed in combination with `#[primary_key]`!",
                ));
            }

            let mut path_value: Option<Path> = None;
            let mut table_value: Option<Ident> = None;

            attr.parse_nested_meta(|meta| {
                match_meta!(match meta {
                    path => {
                        check_duplicate(&path_value, &meta)?;
                        path_value = Some(meta.value()?.parse()?);
                    }
                    table => {
                        check_duplicate(&table_value, &meta)?;
                        table_value = Some(meta.value()?.parse()?);
                    }
                });

                Ok(())
            })?;

            let path_value = Some(path_value.as_ref()
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "PathToTable must be set in `#[referenced_by(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`.",
            ))?
            .to_token_stream()
            .to_string().into());

            let table_value = Some(table_value.as_ref()
            .ok_or_else(|| syn::Error::new_spanned(
                &attr.meta,
                "TableName must be set in `#[referenced_by(table = TableName)]`, e.g. `table = my_table`.",
            ))?
            .to_token_stream()
            .to_string().into());

            referencing_tables.push(ReferencingTable {
                path: path_value.unwrap(),
                table_name: table_value.unwrap(),
            });
        }

        Ok(referencing_tables)
    }
}
