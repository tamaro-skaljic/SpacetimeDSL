use crate::api::{
    Table,
    db::table::SpacetimeDBTable,
    dsl::table::{SpacetimeDSLTable, SpacetimeDSLTableMethods},
};
use quote::format_ident;
use spacetime_bindings_macro_input::table::{ColumnArgs, TableArgs};
use spacetime_bindings_macro_input::{match_meta, util::check_duplicate};
use syn::{DeriveInput, Ident, meta::parser, parse::Parser};

pub(in crate::internal) fn try_parse(
    args: proc_macro2::TokenStream,
    input: &DeriveInput,
    table_args: &TableArgs,
    column_args: &ColumnArgs<'_>,
    plural_name: Option<syn::Ident>,
) -> syn::Result<Table> {
    let rust_struct = crate::internal::rust::table::map_struct(&input);

    let spacetimedb_table = SpacetimeDBTable::map(table_args);

    // Use provided plural_name or parse it from args if not provided
    let plural_name = if let Some(name) = plural_name {
        name
    } else {
        parse_plural_name_from_args(&args)?
            .ok_or_else(|| syn::Error::new(
                proc_macro2::Span::call_site(),
                format_args!("PluralName must be set in `#[dsl(plural_name = PluralName)]`, e.g. `plural_name = {}s`.", spacetimedb_table.singular_name),
            ))?
    };

    let (spacetimedb_table, spacetimedsl_table) =
        SpacetimeDSLTable::try_parse_with_plural_name(column_args, spacetimedb_table, plural_name)?;

    let (spacetimedb_table, columns, internal_columns) = super::column::try_parse(
        &column_args,
        &rust_struct,
        spacetimedb_table,
        &spacetimedsl_table,
    )?;

    let primary_key_column = internal_columns
        .iter()
        .find(|c| c.rust_field_name.to_string().eq(&"id"))
        .expect("should have a primary key");

    let spacetimedsl_methods = SpacetimeDSLTableMethods::try_parse(
        &rust_struct,
        &spacetimedb_table,
        &spacetimedsl_table,
        &columns,
        &internal_columns,
        primary_key_column,
    )?;

    Ok(Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        spacetimedsl_methods,
    })
}

// Parse plural_name from DSL arguments
fn parse_plural_name_from_args(args: &proc_macro2::TokenStream) -> syn::Result<Option<Ident>> {
    use crate::internal::dsl::plural_name;
    
    let mut plural_name_value: Option<Ident> = None;

    parser(|meta| {
        match_meta!(match meta {
            plural_name => {
                check_duplicate(&plural_name_value, &meta)?;
                let value = meta.value()?;
                plural_name_value = Some(value.parse()?);
            }
        });
        Ok(())
    })
    .parse2(args.clone())?;

    Ok(plural_name_value)
}

pub fn rm_rsharp(ident: syn::Ident) -> syn::Ident {
    let mut ident_as_str = ident.to_string();
    if ident_as_str.starts_with("r#") {
        format_ident!("{}", ident_as_str.split_off(2))
    } else {
        ident
    }
}
