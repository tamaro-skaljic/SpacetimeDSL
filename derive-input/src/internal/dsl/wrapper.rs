use super::{path, wrap, wrapped};
use crate::api::dsl::wrapper::{Wrap, Wrapped, WrapperType};
use crate::api::rust::{column::RustField, table::RustStruct};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::{
    match_meta, sats::SatsField, sym::name, util::check_duplicate,
};
use syn::{Error, Ident, Path, Type, parse_str, parse2};

impl WrapperType {
    pub(in crate::internal) fn try_parse(
        rust_struct: &RustStruct,
        rust_field: &RustField,
        field: &SatsField<'_>,
    ) -> syn::Result<Option<WrapperType>> {
        let mut wrapper_type = None;

        for attr in field.original_attrs {
            if attr.meta.path().ne(&wrap) && attr.meta.path().ne(&wrapped) {
                continue;
            }

            if wrapper_type.is_some() {
                return Err(Error::new_spanned(
                    &attr,
                    "Only one of `#[wrap]` or `#[wrapped]` is allowed per column!",
                ));
            }

            let mut wrapper_struct_name_or_path = None;

            match attr.meta.require_path_only() {
                Ok(_) => {
                    if attr.meta.path().eq(&wrap) {
                        wrapper_struct_name_or_path = Some(
                            format!(
                                "{}{}",
                                RenameRule::PascalCase
                                    .apply_to_field(&rust_struct.name.to_string()),
                                RenameRule::PascalCase.apply_to_field(
                                    field.name.as_ref().expect("should have a name")
                                ),
                            )
                            .into(),
                        );
                    } else {
                        return Err(syn::Error::new_spanned(
                            &attr.meta,
                            "PathToWrapperType must be set in `#[wrapped(path = PathToWrapperType)]`, e.g. `path = path::to::my::WrapperType`.",
                        ));
                    }
                }
                Err(_) => {
                    attr.parse_nested_meta(|meta| {
                        match_meta!(match meta {
                            name => {
                                check_duplicate(&wrapper_struct_name_or_path, &meta)?;
                                let wrapper_struct_name: Ident = meta.value()?.parse()?;
                                wrapper_struct_name_or_path = Some(wrapper_struct_name.to_string());
                            }
                            path => {
                                check_duplicate(&wrapper_struct_name_or_path, &meta)?;
                                let wrapper_struct_path: Path = meta.value()?.parse()?;

                                wrapper_struct_name_or_path =
                                    Some(wrapper_struct_path.to_token_stream().to_string());
                            }
                        });
                        Ok(())
                    })?;
                }
            }

            let wrapper_struct_name_or_path =
                wrapper_struct_name_or_path.expect("should have a name or path");
            let wrapped_type_name_or_path = field.ty.to_token_stream().to_string();

            if attr.meta.path().eq(&wrap) {
                let wrapper_struct_name_or_path = format_ident!("{wrapper_struct_name_or_path}");

                let wrapper_impl = get_wrapper_impl(
                    &rust_struct.name,
                    &wrapper_struct_name_or_path,
                    &wrapped_type_name_or_path.clone().into(),
                    &rust_field.name,
                );

                let wrapped_type_name_or_path =
                    parse_str(&wrapped_type_name_or_path).expect("should be parseable");

                wrapper_type = Some(WrapperType::Wrap(Wrap {
                    wrapper_struct_name: wrapper_struct_name_or_path,
                    wrapped_type_name_or_path: wrapped_type_name_or_path,
                    wrapper_impl,
                }));
            } else {
                let wrapper_struct_name_or_path =
                    parse_str(&wrapper_struct_name_or_path).expect("should be parseable");
                let wrapped_type_name_or_path =
                    parse_str(&wrapped_type_name_or_path).expect("should be parseable");

                wrapper_type = Some(WrapperType::Wrapped(Wrapped {
                    wrapper_struct_name_or_path: wrapper_struct_name_or_path,
                    wrapped_type_name_or_path: wrapped_type_name_or_path,
                }));
            }
        }

        Ok(wrapper_type)
    }
}

// TODO: Make sure that the wrapped type implements Default and fail if not. Implement default instead of a custom method.
// TODO: Doc comments on Wrapper Types
fn get_wrapper_impl(
    struct_name: &Ident,
    wrapper_struct_name: &Ident,
    wrapped_type_name_or_path: &Box<str>,
    field_name: &Ident,
) -> TokenStream {
    let wrapped_type: Type = parse_str(wrapped_type_name_or_path).expect(&format!(
        "Expected to parse {wrapped_type_name_or_path} as Type in get_wrapper_impl!"
    ));

    let default_impl;

    if wrapped_type_name_or_path.starts_with("Option <") {
        let wrapped_type_name_or_path: Type = parse_str(
            &wrapped_type_name_or_path
                .replace("Option <", "")
                .replace(">", ""),
        )
        .expect("parsing should have worked");

        default_impl = quote! {
            Some(#wrapped_type_name_or_path::default())
        }
    } else {
        default_impl = quote! {
            #wrapped_type::default()
        }
    }
    quote! {
        #[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType)]
        pub struct #wrapper_struct_name {
            value: #wrapped_type,
        }

        impl From<&#struct_name> for #wrapper_struct_name {
            fn from(value: &#struct_name) -> Self {
                #wrapper_struct_name { value: value.#field_name.clone() }
            }
        }

        impl From<&#struct_name> for Option<#wrapper_struct_name> {
            fn from(value: &#struct_name) -> Option<#wrapper_struct_name> {
                Some(#wrapper_struct_name { value: value.#field_name.clone() })
            }
        }

        impl From<&#wrapper_struct_name> for Option<#wrapper_struct_name> {
            fn from(value: &#wrapper_struct_name) -> Option<#wrapper_struct_name> {
                use spacetimedsl::Wrapper;
                Some(#wrapper_struct_name::new(value.value()))
            }
        }

        impl From<&#wrapper_struct_name> for #wrapper_struct_name {
            fn from(value: &#wrapper_struct_name) -> Self {
                use spacetimedsl::Wrapper;
                #wrapper_struct_name::new(value.value())
            }
        }

        impl spacetimedsl::Wrapper<#wrapped_type, #wrapper_struct_name> for #wrapper_struct_name {
            fn new(value: #wrapped_type) -> Self {
                Self { value }
            }
            fn default() -> Self {
                Self {
                    value: #default_impl,
                }
            }
            fn value(&self) -> #wrapped_type {
                self.value.clone()
            }
        }
    }
}

impl WrapperType {
    pub(in crate::internal) fn map_to_wrapped_type(value: &Wrap) -> Type {
        parse2(value.wrapped_type_name_or_path.to_token_stream()).expect(&format!(
            "Failed to parse {} as Ident in WrapperType::map_to_wrapped_type.",
            &value.wrapped_type_name_or_path.to_token_stream().to_string()
        ))
    }

    pub(in crate::internal) fn map(value: &WrapperType) -> Type {
        match value {
            WrapperType::Wrap(w) => parse_str(&w.wrapper_struct_name.to_token_stream().to_string()).expect(&format!(
                "Failed to parse {} as Ident in WrapperType::map_to_wrapper_type for WrapperType::Wrap.",
                &w.wrapper_struct_name
            )),
            WrapperType::Wrapped(w) => parse_str(&w.wrapper_struct_name_or_path.to_token_stream().to_string()).expect(&format!(
                "Failed to parse {} as Path in WrapperType::map_to_wrapper_type for WrapperType::Wrapped.",
                &w.wrapper_struct_name_or_path.to_token_stream().to_string()
            )),
        }
    }
}

pub(in crate::internal) fn wrapper_type_into_option(
    column_name: &Ident,
    wrapper_type_name_or_path: &Type,
) -> TokenStream {
    let column_option_name = &format_ident!("{column_name}_option");
    quote! {
        let #column_name = #column_name.into();
        let mut #column_option_name = None;
        if #column_name.is_some() {
            #column_option_name = Some(Into::<#wrapper_type_name_or_path>::into(#column_name.expect("value should exist")).value());
        }
        let #column_name = #column_option_name;
    }
}
