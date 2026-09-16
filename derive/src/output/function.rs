use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_input::api::dsl::method::SpacetimeDSLMethod;

use crate::output::{doc_comment, map_args};

// DSL Methods
pub fn build_public(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let parts = MethodParts::new(method);

    let mut output_variants = vec![MethodImplVariant::associated(
        quote! { crate::spacetimedsl::WriteContext },
        quote! { crate::spacetimedsl::DSL<'_, T> },
    )];

    if method.read_context_compatible {
        output_variants.push(MethodImplVariant::associated(
            quote! { crate::spacetimedsl::ReadContext },
            quote! { crate::spacetimedsl::ReadOnlyDSL<'_, T> },
        ));
    }

    build_with_config(
        &parts,
        MethodGenerationConfig {
            doc_variant: MethodImplVariant::associated(
                quote! { crate::spacetimedsl::WriteContext },
                quote! { crate::spacetimedsl::DSL<'_, T> },
            ),
            output_variants,
        },
    )
}

// Execute On Delete Strategies Of [ Referencing Tables | This Table ] After [ One Row | Multiple Rows ] Of [ This | The Referenced ] Table [ Was | Were ] Deleted
pub fn build_internal(method: &SpacetimeDSLMethod) -> syn::Result<TokenStream> {
    let parts = MethodParts::new(method);

    let impl_variant = MethodImplVariant::internal(quote! { crate::spacetimedsl::WriteContext });

    build_with_config(
        &parts,
        MethodGenerationConfig {
            doc_variant: impl_variant.clone(),
            output_variants: vec![impl_variant],
        },
    )
}

struct MethodParts<'a> {
    method: &'a SpacetimeDSLMethod,
    method_args: Vec<TokenStream>,
}

impl<'a> MethodParts<'a> {
    fn new(method: &'a SpacetimeDSLMethod) -> Self {
        Self {
            method,
            method_args: map_args(&method.method_args),
        }
    }
}

#[derive(Clone)]
enum MethodImplTarget {
    InternalDslInternals,
    AssociatedDsl(TokenStream),
}

#[derive(Clone)]
struct MethodImplVariant {
    context_bound: TokenStream,
    target: MethodImplTarget,
}

impl MethodImplVariant {
    fn internal(context_bound: TokenStream) -> Self {
        Self {
            context_bound,
            target: MethodImplTarget::InternalDslInternals,
        }
    }

    fn associated(context_bound: TokenStream, dsl_type: TokenStream) -> Self {
        Self {
            context_bound,
            target: MethodImplTarget::AssociatedDsl(dsl_type),
        }
    }
}

struct MethodGenerationConfig {
    doc_variant: MethodImplVariant,
    output_variants: Vec<MethodImplVariant>,
}

fn build_with_config(
    parts: &MethodParts<'_>,
    config: MethodGenerationConfig,
) -> syn::Result<TokenStream> {
    let doc_comment = doc_comment::doc_comment_with_implementation(
        &parts.method.doc_comment,
        render_impl(parts, &config.doc_variant, None),
    );

    let mut output = TokenStream::new();

    for variant in &config.output_variants {
        output.extend(render_impl(parts, variant, Some(&doc_comment)));
    }

    Ok(output)
}

fn render_impl(
    parts: &MethodParts<'_>,
    variant: &MethodImplVariant,
    doc_comment: Option<&str>,
) -> TokenStream {
    let method_name = &parts.method.method_name;
    let method_args = &parts.method_args;
    let return_type = &parts.method.return_type;
    let method_impl = &parts.method.method_impl;
    // FIXME: We should probably only import one of CtxDbRead or CtxDbWrite per method implementation.
    let method_impl = quote! {
        use ::spacetimedsl::Wrapper;
        use spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};
        #method_impl
    };
    let doc_comment = doc_comment.map(|doc_comment| quote! { #[doc = #doc_comment] });

    match &variant.target {
        MethodImplTarget::InternalDslInternals => {
            let _context_bound = &variant.context_bound;

            quote! {
                impl crate::spacetimedsl::internal::DSLInternals {
                    #doc_comment
                    pub fn #method_name<'a, T: crate::spacetimedsl::WriteContext>(
                        #(#method_args),*
                    ) -> #return_type {
                        #method_impl
                    }
                }
            }
        }
        MethodImplTarget::AssociatedDsl(dsl_type) => {
            let context_bound = &variant.context_bound;

            quote! {
                impl<T: #context_bound> #dsl_type {
                    #[allow(clippy::needless_lifetimes, clippy::too_many_arguments)]
                    #doc_comment
                    pub fn #method_name<'a>(
                        &'a self,
                        #(#method_args),*
                    ) -> #return_type {
                        #method_impl
                    }
                }
            }
        }
    }
}
