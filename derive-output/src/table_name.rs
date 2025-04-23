use proc_macro2::{Ident, TokenTree};
use quote::ToTokens;
use syn::{DeriveInput, parse2};

pub fn get(syntax_tree: &DeriveInput) -> Option<(Ident, Ident)> {
    let mut attribute_containing_singular_table_name = None;
    let mut attribute_containing_plural_table_name = None;

    syntax_tree.attrs.iter().for_each(|attr| {
        let _ = attr.meta.require_list().inspect(|meta_list| {
            if meta_list
                .path
                .to_token_stream()
                .to_string()
                .eq("spacetimedb :: table")
            {
                attribute_containing_singular_table_name = Some(meta_list.tokens.clone());
            } else if meta_list.path.is_ident("plural_table_name") {
                attribute_containing_plural_table_name = Some(meta_list.tokens.clone());
            }
        });
    });

    if attribute_containing_singular_table_name.is_none()
        || attribute_containing_plural_table_name.is_none()
    {
        // TODO: Can't panic / output a proper error because of https://github.com/rust-lang/rust-analyzer/issues/19487
        return None;
    }

    let mut singular_table_name = None;

    for token in attribute_containing_singular_table_name
        .expect("Expected attribute_containing_singular_table_name, found None.")
        .into_iter()
    {
        match token {
            TokenTree::Ident(ident) => {
                if ident.to_string().ne("name")
                    && ident.to_string().ne("public")
                    && ident.to_string().ne("private")
                    && ident.to_string().ne("index")
                {
                    singular_table_name = Some(ident);
                    break;
                }
            }
            _ => {}
        };
    }

    if singular_table_name.is_none() {
        return None;
    }

    let mut plural_table_name: Option<Ident> = None;

    match parse2::<Ident>(
        attribute_containing_plural_table_name
            .expect("Expected attribute_containing_plural_table_name, found none!"),
    ) {
        Ok(name) => {
            plural_table_name = Some(name.clone());
        }
        Err(_) => {}
    }

    if plural_table_name.is_none() {
        return None;
    }

    Some((
        singular_table_name.expect("Expected singular_table_name, found none!"),
        plural_table_name.expect("Expected plural_table_name, found none!"),
    ))
}
