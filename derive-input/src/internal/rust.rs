use crate::api::rust::visibility::RustVisibility;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub mod table;

pub mod column;

impl RustVisibility {
    pub(crate) fn map(value: &syn::Visibility) -> RustVisibility {
        match value {
            syn::Visibility::Public(_) => RustVisibility::Public,
            syn::Visibility::Restricted(vis) => RustVisibility::Restricted(*vis.path.clone()),
            syn::Visibility::Inherited => RustVisibility::Private,
        }
    }
}

impl ToTokens for RustVisibility {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Public => quote! { pub },
            Self::Restricted(path) if path.is_ident("crate") || path.is_ident("super") => {
                quote! { pub(#path) }
            }
            Self::Restricted(path) => quote! { pub(in #path) },
            Self::Private => TokenStream::new(),
        });
    }
}
