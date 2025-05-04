use super::utils::get_table_attribute_macro;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use syn::DeriveInput;

pub(in crate::internal) fn spacetime_bindings_macro_input(
    item: &DeriveInput,
) -> syn::Result<(TableArgs, ColumnArgs)> {
    let input = get_table_attribute_macro(item, "table")?;

    let table_args = TableArgs::parse(input, item)?;

    let (table_args, column_args) = ColumnArgs::parse(table_args, item)?;

    Ok((table_args, column_args))
}
