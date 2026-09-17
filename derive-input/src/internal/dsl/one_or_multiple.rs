use quote::quote;

#[derive(Debug)]
pub(in crate::internal) enum OneOrMultiple {
    One,
    Multiple,
}

impl quote::ToTokens for OneOrMultiple {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            OneOrMultiple::One => crate::api::runtime::one_or_multiple(&quote! { One }),
            OneOrMultiple::Multiple => crate::api::runtime::one_or_multiple(&quote! { Multiple }),
        };
        tokens.extend(variant);
    }
}
