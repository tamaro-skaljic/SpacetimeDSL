use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;

use crate::output::{doc_comment, map_args};

#[derive(Clone, Copy)]
pub(crate) enum ImplTarget {
    Dsl,
    Internals,
}

#[derive(Clone, Copy)]
enum RenderMode<'a> {
    DocExample,
    Emitted(&'a str),
}

#[derive(Clone, Copy)]
enum DslVariant {
    Write,
    ReadOnly,
}

fn build_method_body(method: &SpacetimeDSLMethod) -> TokenStream {
    let additional_paths_to_use = &method.additional_paths_to_use;
    let method_impl = &method.method_impl;

    quote! {
        use ::spacetimedsl::Wrapper;
        use spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};
        #(use #additional_paths_to_use as _;)*
        #method_impl
    }
}

fn render_impl(
    method: &SpacetimeDSLMethod,
    method_args: &[TokenStream],
    target: ImplTarget,
    mode: RenderMode<'_>,
    dsl_variant: Option<DslVariant>,
) -> TokenStream {
    let method_name = &method.method_name;
    let return_type = &method.return_type;
    let method_body = build_method_body(method);
    let impl_header = impl_header_for(target, dsl_variant);
    let method_generics = method_generics_for(target);
    let receiver = receiver_for(target);
    let attributes = attributes_for(target, mode);

    quote! {
        #impl_header {
            #attributes
            pub fn #method_name #method_generics(
                #receiver
                #(#method_args),*
            ) -> #return_type {
                #method_body
            }
        }
    }
}

fn impl_header_for(target: ImplTarget, dsl_variant: Option<DslVariant>) -> TokenStream {
    match target {
        ImplTarget::Dsl => {
            let (context_bound, dsl_type) = match dsl_variant.expect("DSL variant should exist") {
                DslVariant::Write => (
                    quote! { crate::spacetimedsl::WriteContext },
                    quote! { crate::spacetimedsl::DSL<'_, T> },
                ),
                DslVariant::ReadOnly => (
                    quote! { crate::spacetimedsl::ReadContext },
                    quote! { crate::spacetimedsl::ReadOnlyDSL<'_, T> },
                ),
            };

            quote! {
                impl<T: #context_bound> #dsl_type
            }
        }
        ImplTarget::Internals => quote! {
            impl crate::spacetimedsl::internal::DSLInternals
        },
    }
}

fn method_generics_for(target: ImplTarget) -> TokenStream {
    match target {
        ImplTarget::Dsl => quote! { <'a> },
        ImplTarget::Internals => quote! { <'a, T: crate::spacetimedsl::WriteContext> },
    }
}

fn receiver_for(target: ImplTarget) -> TokenStream {
    match target {
        ImplTarget::Dsl => quote! { &'a self, },
        ImplTarget::Internals => TokenStream::default(),
    }
}

fn attributes_for(target: ImplTarget, mode: RenderMode<'_>) -> TokenStream {
    match (target, mode) {
        (ImplTarget::Dsl, RenderMode::DocExample) => quote! {
            #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
        },
        (ImplTarget::Dsl, RenderMode::Emitted(doc_comment)) => quote! {
            #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
            #[doc = #doc_comment]
        },
        (ImplTarget::Internals, RenderMode::DocExample) => TokenStream::default(),
        (ImplTarget::Internals, RenderMode::Emitted(doc_comment)) => quote! {
            #[doc = #doc_comment]
        },
    }
}

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build(method: &SpacetimeDSLMethod, target: ImplTarget) -> syn::Result<TokenStream> {
    let method_args = map_args(&method.method_args);
    let doc_example_variant = match target {
        ImplTarget::Dsl => Some(DslVariant::Write),
        ImplTarget::Internals => None,
    };
    let doc_comment = doc_comment::doc_comment_with_implementation(
        &method.doc_comment,
        render_impl(
            method,
            &method_args,
            target,
            RenderMode::DocExample,
            doc_example_variant,
        ),
    );

    let primary_variant = match target {
        ImplTarget::Dsl => Some(DslVariant::Write),
        ImplTarget::Internals => None,
    };
    let mut output = render_impl(
        method,
        &method_args,
        target,
        RenderMode::Emitted(&doc_comment),
        primary_variant,
    );

    if matches!(target, ImplTarget::Dsl) && method.read_context_compatible {
        output.extend(render_impl(
            method,
            &method_args,
            target,
            RenderMode::Emitted(&doc_comment),
            Some(DslVariant::ReadOnly),
        ));
    }

    Ok(output)
}
