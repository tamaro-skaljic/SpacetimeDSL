use crate::api::dsl::column::{Wrap, Wrapped, WrapperType};
use crate::internal::dsl::table::path;
use ident_case::RenameRule;
use quote::ToTokens;
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

            if attr.meta.path().eq(&wrap) {
                wrapper_type = Some(WrapperType::Wrap(Wrap {
                    wrapper_struct_name: wrapper_struct_name_or_path,
                    wrapped_type_name_or_path,
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
