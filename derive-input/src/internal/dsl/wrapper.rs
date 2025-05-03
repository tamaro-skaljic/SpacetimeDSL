use super::getter::get_getter_method_name;
use crate::api::dsl::wrapper::{Wrap, Wrapped, WrapperType};
use crate::internal::dsl::table::path;
use ident_case::RenameRule;
use quote::ToTokens;
use quote::quote;
use spacetime_bindings_macro_input::{
    match_meta,
    sats::SatsField,
    sym::{Symbol, name},
    symbol,
    util::check_duplicate,
};
use syn::{Ident, Path};

impl WrapperType {
    pub(in crate::internal) fn try_parse(
        item: &syn::DeriveInput,
        field: &SatsField<'_>,
    ) -> syn::Result<Option<WrapperType>> {
        let mut wrapper_type = None;

        for attr in field.original_attrs {
            if attr.meta.path().ne(&wrap) && attr.meta.path().ne(&wrapped) {
                continue;
            }

            if wrapper_type.is_some() {
                return Err(syn::Error::new_spanned(
                    &attr,
                    "Only one of `#[wrap]` or `#[wrapped]` is allowed per column!",
                ));
            }

            let mut wrapper_struct_name_or_path: Option<Box<str>> = None;

            attr.parse_nested_meta(|meta| {
                match_meta!(match meta {
                    name => {
                        check_duplicate(&wrapper_struct_name_or_path, &meta)?;
                        let wrapper_struct_name: Ident = meta.value()?.parse()?;
                        wrapper_struct_name_or_path = Some(wrapper_struct_name.to_string().into())
                    }
                    path => {
                        check_duplicate(&wrapper_struct_name_or_path, &meta)?;
                        let wrapper_struct_path: Path = meta.value()?.parse()?;
                        wrapper_struct_name_or_path =
                            Some(wrapper_struct_path.to_token_stream().to_string().into())
                    }
                });
                Ok(())
            })?;

            if wrapper_struct_name_or_path.is_none() {
                if attr.meta.path().eq(&wrap) {
                    wrapper_struct_name_or_path = Some(
                        format!(
                            "{}{}",
                            RenameRule::PascalCase.apply_to_field(item.ident.to_string()),
                            RenameRule::PascalCase.apply_to_field(field.name.as_ref().unwrap()),
                        )
                        .into(),
                    );
                } else {
                    return Err(syn::Error::new_spanned(
                        &attr.meta,
                        "WrapperPath must be set in `#[wrapped(WrapperPath)]`, e.g. `path::to::TableId`.",
                    ));
                }
            }

            let wrapper_struct_name_or_path = wrapper_struct_name_or_path.unwrap();
            let wrapped_type_name_or_path = field.ty.to_token_stream().to_string().into();
            let wrapper_impl = get_wrapper_impl(
                &item.ident.to_string().into(),
                &wrapper_struct_name_or_path,
                &wrapped_type_name_or_path,
                &get_getter_method_name(&field.name.as_ref().unwrap().to_string().into_boxed_str()),
            );

            if attr.meta.path().eq(&wrap) {
                wrapper_type = Some(WrapperType::Wrap(Wrap {
                    wrapper_struct_name: wrapper_struct_name_or_path,
                    wrapped_type_name_or_path,
                    wrapper_impl,
                }));
            } else {
                wrapper_type = Some(WrapperType::Wrapped(Wrapped {
                    wrapper_struct_name_or_path,
                    wrapped_type_name_or_path,
                }));
            }
        }

        Ok(wrapper_type)
    }
}

symbol!(wrap);
symbol!(wrapped);

fn get_wrapper_impl(
    struct_name: &Box<str>,
    wrapper_struct_name: &Box<str>,
    wrapped_type_name_or_path: &Box<str>,
    getter_name: &Box<str>,
) -> Box<str> {
    quote! {
        #[derive(Clone, Debug, PartialEq, spacetimedb::SpacetimeType)]
        pub struct #wrapper_struct_name {
            value: #wrapped_type_name_or_path,
        }

        impl From<&#struct_name> for #wrapper_struct_name {
            fn from(value: &#struct_name) -> Self {
                value.#getter_name()
            }
        }

        impl From<&#struct_name> for Option<#wrapper_struct_name> {
            fn from(value: &#struct_name) -> Option<#wrapper_struct_name> {
                Some(value.#getter_name())
            }
        }

        impl spacetimedsl::Wrapper<#wrapped_type_name_or_path, #wrapper_struct_name> for #wrapper_struct_name {
            fn new(value: #wrapped_type_name_or_path) -> Self {
                Self { value }
            }
            fn default() -> Self {
                Self {
                    value: #wrapped_type_name_or_path::default(),
                }
            }
            fn value(&self) -> #wrapped_type_name_or_path {
                self.value.clone()
            }
        }
    }.to_string().into()
}
