use crate::api::dsl::table::DSLTable;
use quote::ToTokens;
use spacetime_bindings_macro_input::match_meta;
use spacetime_bindings_macro_input::sym::Symbol;
use spacetime_bindings_macro_input::symbol;
use spacetime_bindings_macro_input::util::check_duplicate;
use syn::Ident;
use syn::parse::Parser;

impl DSLTable {
    pub(in crate::internal) fn try_parse(args: &syn::Attribute) -> syn::Result<Option<DSLTable>> {
        let mut name_plural: Option<Box<Ident>> = None;

        syn::meta::parser(|meta| {
            match_meta!(match meta {
                plural_name => {
                    check_duplicate(&name_plural, &meta)?;
                    let value = meta.value()?;
                    name_plural = Some(value.parse()?);
                }
            });
            Ok(())
        })
        .parse2(args.to_token_stream())?;

        if name_plural.is_none() {
            return Ok(None);
        }

        let name_plural = name_plural.unwrap().to_token_stream().to_string().into();

        // Is set to true later if a column is mutable
        let is_mutable = false;
        // Is set to true later if the column exists
        let has_created_at_column = false;
        // Is set to true later if the column exists
        let has_modified_at_column = false;
        // Is set to Some(T) later after all columns are parsed.
        let dsl_methods = None;

        Ok(Some(DSLTable {
            plural_name: name_plural,
            is_mutable,
            has_created_at_column,
            has_modified_at_column,
            dsl_methods,
        }))
    }
}

symbol!(plural_name);
