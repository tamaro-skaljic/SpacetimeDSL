use {
    super::{create_wrapper, use_wrapper},
    crate::{
        api::{
            dsl::wrapper::{CreatedWrapper, UsedWrapper, WrapperType},
            runtime,
            rust::{column::RustField, table::RustStruct},
            spacetimedb,
        },
        internal::{column::ColumnTypeKind, error},
    },
    ident_case::RenameRule,
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    spacetime_bindings_macro_input::sats::SatsField,
    syn::{Ident, Path},
};

/// What `#[create_wrapper]` or `#[use_wrapper(...)]` on a column says, before anything is
/// generated from it.
enum WrapperAttribute {
    /// `#[create_wrapper]` or `#[create_wrapper(Name)]`: the name of the type to generate.
    Created { name: Ident },
    /// `#[use_wrapper(path)]`: the wrapper type another column created.
    Used { path: Path },
}

impl WrapperAttribute {
    /// The wrapper attribute of a field, if it has one. A field may have only one.
    fn try_parse(
        rust_struct: &RustStruct,
        field: &SatsField<'_>,
    ) -> syn::Result<Option<WrapperAttribute>> {
        let mut wrapper_attribute = None;

        for attr in field.original_attrs {
            let is_create_wrapper = attr.path() == create_wrapper;

            if !is_create_wrapper && attr.path() != use_wrapper {
                continue;
            }

            if wrapper_attribute.is_some() {
                return Err(error::multiple_wrappers(attr));
            }

            let has_arguments = attr.meta.require_path_only().is_err();

            wrapper_attribute = Some(match (is_create_wrapper, has_arguments) {
                (true, false) => WrapperAttribute::Created {
                    name: format_ident!(
                        "{}{}",
                        RenameRule::PascalCase.apply_to_field(rust_struct.name.to_string()),
                        RenameRule::PascalCase
                            .apply_to_field(field.name.as_ref().expect("should have a name")),
                    ),
                },
                (true, true) => WrapperAttribute::Created {
                    name: attr
                        .meta
                        .require_list()?
                        .parse_args()
                        .map_err(|_| error::invalid_create_wrapper_name(&attr.meta))?,
                },
                (false, false) => return Err(error::missing_use_wrapper_path(&attr.meta)),
                (false, true) => WrapperAttribute::Used {
                    path: attr
                        .meta
                        .require_list()?
                        .parse_args()
                        .map_err(|_| error::invalid_use_wrapper_path(&attr.meta))?,
                },
            });
        }

        Ok(wrapper_attribute)
    }
}

impl WrapperType {
    pub(crate) fn try_parse(
        rust_struct: &RustStruct,
        rust_field: &RustField,
        field: &SatsField<'_>,
    ) -> syn::Result<Option<WrapperType>> {
        let Some(wrapper_attribute) = WrapperAttribute::try_parse(rust_struct, field)? else {
            return Ok(None);
        };

        let wrapped_type_name_or_path = rust_field.type_name_or_path.clone();

        Ok(Some(match wrapper_attribute {
            WrapperAttribute::Created { name } => WrapperType::Created(CreatedWrapper {
                wrapper_impl: created_wrapper_impl(
                    &rust_struct.name,
                    &name,
                    &wrapped_type_name_or_path,
                    &rust_field.name,
                    ColumnTypeKind::of(&wrapped_type_name_or_path) == ColumnTypeKind::UUID,
                ),
                wrapper_struct_name: name,
                wrapped_type_name_or_path,
            }),
            WrapperAttribute::Used { path } => WrapperType::Used(UsedWrapper {
                wrapper_struct_name_or_path: path,
                wrapped_type_name_or_path,
            }),
        }))
    }
}

/// The wrapper type `#[create_wrapper]` generates, with its trait implementations.
fn created_wrapper_impl(
    struct_name: &Ident,
    wrapper_struct_name: &Ident,
    wrapped_type: &Path,
    field_name: &Ident,
    wraps_uuid: bool,
) -> TokenStream {
    let wrapper_struct_name_as_str = wrapper_struct_name.to_string();

    let wrapper_trait = runtime::wrapper_trait(&wrapped_type);
    let wrapper_trait_import = runtime::wrapper_trait_import();
    let spacetimetype_derive = spacetimedb::spacetimetype_derive();

    // `Uuid` has no `Default`, so its wrapper generates a fresh value instead.
    let constructors = if wraps_uuid {
        uuid_wrapper_constructors(wrapper_struct_name)
    } else {
        quote! {
            impl Default for #wrapper_struct_name {
                fn default() -> #wrapper_struct_name {
                    #wrapper_struct_name { value: Default::default() }
                }
            }
        }
    };

    quote! {
        #[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash, #spacetimetype_derive)]
        pub struct #wrapper_struct_name {
            value: #wrapped_type,
        }

        #constructors

        impl std::fmt::Display for #wrapper_struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ id: {:?} }}", #wrapper_struct_name_as_str, self.value)
            }
        }

        impl #wrapper_trait for #wrapper_struct_name {
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
                #wrapper_trait_import
                Some(#wrapper_struct_name::new(value.value()))
            }
        }

        impl From<&#wrapper_struct_name> for #wrapper_struct_name {
            fn from(value: &#wrapper_struct_name) -> Self {
                #wrapper_trait_import
                #wrapper_struct_name::new(value.value())
            }
        }
    }
}

fn uuid_wrapper_constructors(wrapper_struct_name: &Ident) -> TokenStream {
    let dsl_reference_type = runtime::dsl_reference_type_with_any_write_context();
    let new_uuid_trait = runtime::new_uuid_trait();
    let result_type = runtime::error_result_type(&quote! { Self });

    quote! {
        impl #wrapper_struct_name {
            /// Generate a random UUID v4.
            pub fn v4(dsl: #dsl_reference_type) -> #result_type {
                Ok(Self { value: #new_uuid_trait::new_uuid_v4(dsl.ctx())? })
            }

            /// Generate a UUID v7, which sorts in the order the values were generated.
            pub fn v7(dsl: #dsl_reference_type) -> #result_type {
                Ok(Self { value: #new_uuid_trait::new_uuid_v7(dsl.ctx())? })
            }
        }
    }
}

impl WrapperType {
    /// The wrapper type: a created wrapper's name as a one-segment path, or the path
    /// `#[use_wrapper(...)]` names.
    pub(crate) fn wrapper_path(&self) -> Path {
        match self {
            WrapperType::Created(created_wrapper) => {
                Path::from(created_wrapper.wrapper_struct_name.clone())
            }
            WrapperType::Used(used_wrapper) => used_wrapper.wrapper_struct_name_or_path.clone(),
        }
    }

    /// The column type the wrapper type wraps.
    pub(crate) fn wrapped_type(&self) -> &Path {
        match self {
            WrapperType::Created(created_wrapper) => &created_wrapper.wrapped_type_name_or_path,
            WrapperType::Used(used_wrapper) => &used_wrapper.wrapped_type_name_or_path,
        }
    }

    /// The wrapper's own name without its module path, as doc comments name it.
    pub(crate) fn struct_name(&self) -> Ident {
        match self {
            WrapperType::Created(created_wrapper) => created_wrapper.wrapper_struct_name.clone(),
            WrapperType::Used(used_wrapper) => used_wrapper
                .wrapper_struct_name_or_path
                .segments
                .last()
                .expect("A parsed path always has a last segment")
                .ident
                .clone(),
        }
    }
}

pub fn map_wrapper_type_option_to_wrapped_type_option(
    column_name: &Ident,
    wrapper_type_name_or_path: &Path,
) -> TokenStream {
    quote! {
        let #column_name = #column_name.map(|value| Into::<#wrapper_type_name_or_path>::into(value).value());
    }
}
