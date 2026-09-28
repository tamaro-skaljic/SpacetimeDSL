//! The contract between the code SpacetimeDSL generates and the `spacetimedb` crate it runs
//! against, the counterpart of [`crate::api::runtime`].
//!
//! Every `::spacetimedb::…` path any generator emits is written here exactly once, so a
//! renamed or relocated SpacetimeDB item is a change to this file rather than a text hunt
//! through `quote!` bodies that no compiler checks. The leading `::` names the crate even
//! where a module of the user's crate is called `spacetimedb`.
//!
//! These are plain token constructors: they splice already-built token streams and take no
//! decisions. They must not grow branching, or they become a second generator.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// `TryInsertError::#variant`, the error a failed `try_insert` returns.
pub fn try_insert_error(variant: &impl ToTokens) -> TokenStream {
    quote! {
        ::spacetimedb::TryInsertError::#variant
    }
}

/// `SpacetimeType`, the derive a generated wrapper type is given so that it can be a column
/// type.
pub fn spacetimetype_derive() -> TokenStream {
    quote! {
        ::spacetimedb::SpacetimeType
    }
}

/// `use ::spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};`, the import every method body
/// opens with, so it can reach the tables of the database.
pub fn table_traits_import() -> TokenStream {
    quote! {
        use ::spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};
    }
}

/// `Uuid::NIL`, the UUID which references no row.
pub fn uuid_nil() -> TokenStream {
    quote! {
        ::spacetimedb::Uuid::NIL
    }
}
