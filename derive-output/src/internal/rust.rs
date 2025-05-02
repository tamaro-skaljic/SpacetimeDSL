use crate::api::rust::{RustField, RustStruct, RustVisibility};
use quote::ToTokens;
use syn::Visibility;

impl RustStruct {
    pub(in crate::internal) fn map(input: &syn::DeriveInput) -> RustStruct {
        let visibility = RustVisibility::map(&input.vis);
        let name = input.ident.to_string().into();

        RustStruct { visibility, name }
    }
}

impl RustField {
    pub(in crate::internal) fn map(
        field: &spacetime_bindings_macro_input::sats::SatsField<'_>,
    ) -> RustField {
        let visibility = RustVisibility::map(field.vis);
        let name = field.ident.as_ref().unwrap().to_string().into();
        let r#type = field.ty.to_token_stream().to_string().into();

        RustField {
            visibility,
            name,
            type_name_or_path: r#type,
        }
    }
}

impl RustVisibility {
    fn map(value: &Visibility) -> RustVisibility {
        match value {
            Visibility::Public(_) => RustVisibility::Public,
            Visibility::Restricted(vis) => {
                RustVisibility::Restricted(vis.path.to_token_stream().to_string().into())
            }
            Visibility::Inherited => RustVisibility::Private,
        }
    }
}
