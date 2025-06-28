use crate::api::rust::visibility::RustVisibility;
use quote::ToTokens;
use std::fmt;

pub mod table;

pub mod column;

impl RustVisibility {
    pub(in crate::internal) fn map(value: &syn::Visibility) -> RustVisibility {
        match value {
            syn::Visibility::Public(_) => RustVisibility::Public,
            syn::Visibility::Restricted(vis) => RustVisibility::Restricted(*vis.path.clone()),
            syn::Visibility::Inherited => RustVisibility::Private,
        }
    }
}

impl fmt::Display for RustVisibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => {
                write!(f, "pub")
            }
            Self::Restricted(str) => {
                let str = str.to_token_stream().to_string();

                match str.as_str() {
                    "crate" => {
                        write!(f, "pub (crate)")
                    }
                    "super" => {
                        write!(f, "pub (super)")
                    }
                    str => {
                        write!(f, "pub (in {str})")
                    }
                }
            }
            Self::Private => {
                write!(f, "")
            }
        }
    }
}
