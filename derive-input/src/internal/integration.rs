use {
    crate::{api::attribute::is_table_attribute, internal::error},
    spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs},
    syn::{DeriveInput, Ident},
};

/// The `#[table]` attribute a `#[dsl]` attribute belongs to.
///
/// With several `#[table]` attributes on the struct, `#[dsl(table = <accessor>)]` has to
/// name it. With one, the selector is optional, but has to name that one when given. A
/// singleton has to have exactly one.
pub fn select_table_attribute<'a>(
    item: &'a DeriveInput,
    table_selector: Option<&Ident>,
    is_singleton: bool,
) -> syn::Result<(TableArgs, ColumnArgs<'a>)> {
    let all_tables = get_all_table_attributes(item)?;

    if is_singleton && all_tables.len() != 1 {
        return Err(error::singleton_without_exactly_one_table_attribute(
            &item.ident,
            all_tables.len(),
        ));
    }

    let accessors = || -> Vec<&Ident> {
        all_tables
            .iter()
            .map(|(table_args, _)| &table_args.accessor)
            .collect()
    };

    let position = match table_selector {
        Some(table_selector) => all_tables
            .iter()
            .position(|(table_args, _)| table_args.accessor == *table_selector)
            .ok_or_else(|| error::table_selector_names_no_table(table_selector, &accessors()))?,
        None if all_tables.len() == 1 => 0,
        None => return Err(error::table_selector_missing(&item.ident, &accessors())),
    };

    Ok(all_tables
        .into_iter()
        .nth(position)
        .expect("the position of the selected table was just found"))
}

fn get_all_table_attributes<'a>(
    input: &'a DeriveInput,
) -> syn::Result<Vec<(TableArgs, ColumnArgs<'a>)>> {
    let table_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| is_table_attribute(attr))
        .filter_map(|attr| attr.meta.require_list().ok())
        .map(|list| list.tokens.clone())
        .collect();

    if table_attrs.is_empty() {
        return Err(error::missing_table_attribute(&input.ident));
    }

    let mut results = vec![];
    for table_attr in table_attrs {
        let table_args = TableArgs::parse(table_attr, input)?;
        let (table_args, column_args) = ColumnArgs::parse(table_args, input)?;
        results.push((table_args, column_args));
    }

    Ok(results)
}
