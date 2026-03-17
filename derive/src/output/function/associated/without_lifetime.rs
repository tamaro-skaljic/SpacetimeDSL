use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;

// create, get_one_option, update, delete_one
pub fn build(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let mut doc_comment = String::new();
    doc_comment.push_str(&method.doc_comment);

    let trait_name = &method.trait_name;
    let trait_dep_paths = &method.trait_dep_paths;
    let method_name = &method.method_name;

    let method_args = crate::output::map_args(&method.method_args);

    let return_type = &method.return_type;
    let method_impl = &method.method_impl;

    doc_comment = crate::output::function::add_impl_doc(
        trait_name,
        trait_dep_paths,
        method_name,
        &method_args,
        return_type,
        method_impl,
        doc_comment,
    );

    let trait_comment = format!("See [`Self::{method_name}`] for details.");

    let mut tokens = quote! {
        #[doc = #trait_comment]
        pub trait #trait_name {
            #[allow(clippy::too_many_arguments)]
            #[doc=#doc_comment]
            fn #method_name(
                &self,
                #(#method_args),*
            ) -> #return_type;
        }

        impl<T: spacetimedsl::WriteContext> #trait_name for spacetimedsl::DSL<'_, T> {
            #[allow(clippy::too_many_arguments)]
            fn #method_name(
                &self,
                #(#method_args),*
            ) -> #return_type {
                use spacetimedsl::DSLContext;
                #(use #trait_dep_paths as _;)*
                use spacetimedsl::Wrapper;
                use spacetimedb::{DbContext, Table as _};
                #method_impl
            }
        }
    };

    if method.read_context_compatible {
        tokens.extend(quote! {
            impl<T: spacetimedsl::ReadContext> #trait_name for spacetimedsl::ReadOnlyDSL<'_, T> {
                #[allow(clippy::too_many_arguments)]
                fn #method_name(
                    &self,
                    #(#method_args),*
                ) -> #return_type {
                    use spacetimedsl::ReadOnlyDSLContext;
                    #(use #trait_dep_paths as _;)*
                    use spacetimedsl::Wrapper;
                    use spacetimedb::{DbContext, Table as _};
                    #method_impl
                }
            }
        });
    }

    Ok(tokens)
}
