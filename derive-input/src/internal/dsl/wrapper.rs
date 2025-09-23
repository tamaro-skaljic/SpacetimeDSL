use super::{create_wrapper, use_wrapper};
use crate::api::dsl::wrapper::{CreatedWrapper, UsedWrapper, WrapperType};
use crate::api::rust::{column::RustField, table::RustStruct};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::quote;
use quote::{ToTokens, format_ident};
use spacetime_bindings_macro_input::sats::SatsField;
use syn::{Error, Ident, Path, Type, parse_str, parse2};

impl WrapperType {
    pub(in crate::internal) fn try_parse(
        rust_struct: &RustStruct,
        rust_field: &RustField,
        field: &SatsField<'_>,
    ) -> syn::Result<Option<WrapperType>> {
        let mut wrapper_type = None;

        for attr in field.original_attrs {
            if attr.meta.path().ne(&create_wrapper) && attr.meta.path().ne(&use_wrapper) {
                continue;
            }

            if wrapper_type.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "Only one of `#[create_wrapper]` or `#[use_wrapper]` is allowed per column!",
                ));
            }

            let wrapper_struct_name_or_path;

            match attr.meta.require_path_only() {
                Ok(_) => {
                    if attr.meta.path().eq(&create_wrapper) {
                        wrapper_struct_name_or_path = Some(format!(
                            "{}{}",
                            RenameRule::PascalCase.apply_to_field(rust_struct.name.to_string()),
                            RenameRule::PascalCase
                                .apply_to_field(field.name.as_ref().expect("should have a name")),
                        ));
                    } else {
                        return Err(syn::Error::new_spanned(
                            &attr.meta,
                            "PathToWrapperType must be set in `#[use_wrapper(PathToWrapperType)]`, e.g. `EntityId` or `crate::entity::EntityId`.",
                        ));
                    }
                }
                Err(_) => {
                    if attr.meta.path().eq(&create_wrapper) {
                        let ident: Ident = attr.meta.require_list()?.parse_args()
                            .map_err(|_| syn::Error::new_spanned(
                                    &attr.meta,
                                    "Failed to parse NameForWrapperType in `#[create_wrapper(NameForWrapperType)]`. Expected a valid Rust ident like `EntityId`.",
                                ))?;
                        wrapper_struct_name_or_path = Some(ident.to_string());
                    } else {
                        let wrapper_struct_path: Path = attr.meta.require_list()?.parse_args()
                            .map_err(|_| syn::Error::new_spanned(
                                &attr.meta,
                                "Failed to parse PathToWrapperType in `#[use_wrapper(PathToWrapperType)]`. Expected a valid Rust path like `EntityId` or `crate::entity::EntityId`.",
                            ))?;
                        wrapper_struct_name_or_path =
                            Some(wrapper_struct_path.to_token_stream().to_string());
                    }
                }
            }

            let wrapper_struct_name_or_path =
                wrapper_struct_name_or_path.expect("should have a name or path");
            let wrapped_type_name_or_path = field.ty.to_token_stream().to_string();

            if attr.meta.path().eq(&create_wrapper) {
                let wrapper_struct_name_or_path = format_ident!("{wrapper_struct_name_or_path}");

                let wrapper_impl = get_wrapper_impl(
                    &rust_struct.name,
                    &wrapper_struct_name_or_path,
                    &wrapped_type_name_or_path,
                    &rust_field.name,
                );

                let wrapped_type_name_or_path =
                    parse_str(&wrapped_type_name_or_path).expect("should be parseable");

                wrapper_type = Some(WrapperType::Created(CreatedWrapper {
                    wrapper_struct_name: wrapper_struct_name_or_path,
                    wrapped_type_name_or_path,
                    wrapper_impl,
                }));
            } else {
                let wrapper_struct_name_or_path =
                    parse_str(&wrapper_struct_name_or_path).expect("should be parseable");
                let wrapped_type_name_or_path =
                    parse_str(&wrapped_type_name_or_path).expect("should be parseable");

                wrapper_type = Some(WrapperType::Used(UsedWrapper {
                    wrapper_struct_name_or_path,
                    wrapped_type_name_or_path,
                }));
            }
        }

        Ok(wrapper_type)
    }
}

// TODO: Doc comments on Wrapper Types
fn get_wrapper_impl(
    struct_name: &Ident,
    wrapper_struct_name: &Ident,
    wrapped_type_name_or_path: &str,
    field_name: &Ident,
) -> TokenStream {
    let wrapped_type: Type = parse_str(wrapped_type_name_or_path).unwrap_or_else(|_| {
        panic!("Expected to parse {wrapped_type_name_or_path} as Type in get_wrapper_impl!")
    });

    let wrapper_struct_name_as_str = wrapper_struct_name.to_string();

    quote! {
        #[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType)]
        pub struct #wrapper_struct_name {
            value: #wrapped_type,
        }

        impl Default for #wrapper_struct_name {
            fn default() -> #wrapper_struct_name {
                #wrapper_struct_name { value: Default::default() }
            }
        }

        impl std::fmt::Display for #wrapper_struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ id: {:?} }}", #wrapper_struct_name_as_str, self.value)
            }
        }

        impl spacetimedsl::Wrapper<#wrapped_type, #wrapper_struct_name> for #wrapper_struct_name {
            fn new(value: #wrapped_type) -> Self {
                Self { value }
            }
            fn value(&self) -> #wrapped_type {
                self.value.clone()
            }
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
    }
}

impl WrapperType {
    pub(in crate::internal) fn map_to_wrapped_type(value: &WrapperType) -> Type {
        let wrapped_type_name_or_path = match value {
            WrapperType::Created(created_wrapper) => &created_wrapper.wrapped_type_name_or_path,
            WrapperType::Used(used_wrapper) => &used_wrapper.wrapped_type_name_or_path,
        };

        parse2(wrapped_type_name_or_path.to_token_stream()).unwrap_or_else(|_| {
            panic!(
                "Failed to parse {} as Ident in WrapperType::map_to_wrapped_type.",
                &wrapped_type_name_or_path.to_token_stream().to_string()
            )
        })
    }

    pub(in crate::internal) fn map(value: &WrapperType) -> Type {
        match value {
            WrapperType::Created(w) => parse_str(&w.wrapper_struct_name.to_token_stream().to_string()).unwrap_or_else(|_| panic!("Failed to parse {} as Ident in WrapperType::map_to_wrapper_type for WrapperType::Wrap.",
                &w.wrapper_struct_name)),
            WrapperType::Used(w) => parse_str(&w.wrapper_struct_name_or_path.to_token_stream().to_string()).unwrap_or_else(|_| panic!("Failed to parse {} as Path in WrapperType::map_to_wrapper_type for WrapperType::Wrapped.",
                &w.wrapper_struct_name_or_path.to_token_stream().to_string())),
        }
    }
}

pub(in crate::internal) fn map_wrapper_type_option_to_wrapped_type_option(
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
