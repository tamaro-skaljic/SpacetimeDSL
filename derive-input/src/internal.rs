pub(crate) mod integration;

mod table;

mod column;

mod rust;

mod db;

mod dsl;

pub(crate) fn try_parse(
    args: proc_macro2::TokenStream,
    input: &syn::DeriveInput,
) -> syn::Result<crate::api::Table> {
    // Parse plural_name from DSL arguments - it's required
    let plural_name = parse_plural_name_from_args(&args)?
        .ok_or_else(|| syn::Error::new(
            proc_macro2::Span::call_site(),
            "PluralName must be set in `#[dsl(plural_name = PluralName)]`",
        ))?;
    
    // Pass plural_name to integration for intelligent table selection
    let (table_args, column_args) = integration::spacetime_bindings_macro_input(input, Some(&plural_name))?;
    
    // Pass the parsed plural_name to avoid re-parsing
    table::try_parse(args, input, &table_args, &column_args, plural_name)
}

// Parse plural_name from DSL arguments
fn parse_plural_name_from_args(args: &proc_macro2::TokenStream) -> syn::Result<Option<syn::Ident>> {
    use spacetime_bindings_macro_input::{match_meta, util::check_duplicate};
    use syn::{meta::parser, parse::Parser};
    use dsl::plural_name;
    
    let mut plural_name_value: Option<syn::Ident> = None;

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
