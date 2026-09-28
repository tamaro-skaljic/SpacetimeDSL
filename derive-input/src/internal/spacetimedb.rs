//! The `::spacetimedb::…` paths the generators emit inside the code they generate, the
//! counterpart of [`crate::api::runtime`] for the `spacetimedb` crate.
//!
//! Every such path is written once: here, or in [`crate::api::spacetimedb`] when a crate
//! rendering the generated code has to emit it too. A renamed or relocated SpacetimeDB item
//! is then a change to these two files rather than a text hunt through `quote!` bodies that
//! no compiler checks. The leading `::` names the crate even where a module of the user's
//! crate is called `spacetimedb`.
//!
//! These are plain token constructors: they splice already-built token streams and take no
//! decisions. They must not grow branching, or they become a second generator.

use {
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
};

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

/// `Uuid::NIL`, the UUID which references no row.
pub fn uuid_nil() -> TokenStream {
    quote! {
        ::spacetimedb::Uuid::NIL
    }
}
