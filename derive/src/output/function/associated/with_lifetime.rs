use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;

// get_all, get_count, get_many, delete_many
pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = &method.trait_name;
    let paths_of_traits_to_extend = &method.paths_of_traits_to_extend;
    let method_name = &method.method_name;

    let method_args = crate::output::map_args(&method.method_args);

    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    doc_comment = crate::output::function::add_impl_doc(
        trait_name,
        paths_of_traits_to_extend,
        method_name,
        &method_args,
        return_type,
        method_impl,
        doc_comment,
    );

    let trait_comment = format!("See [`Self::{method_name}`] for details.");
    let method = quote! {
        #[doc = #trait_comment]
        pub trait #trait_name: #(#paths_of_traits_to_extend)+* {
            #[doc=#doc_comment]
            fn #method_name<'a>(
                &'a self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #method_impl
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    };

    Ok(method)
}
