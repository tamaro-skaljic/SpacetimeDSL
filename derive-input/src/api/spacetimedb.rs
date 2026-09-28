//! The `::spacetimedb::…` path a crate rendering the DSL methods emits around their bodies,
//! the counterpart of [`crate::api::runtime`] for the `spacetimedb` crate.
//!
//! The leading `::` names the crate even where a module of the user's crate is called
//! `spacetimedb`. The paths the generators emit inside the bodies are not public.

use {proc_macro2::TokenStream, quote::quote};

/// `use ::spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};`, the import every method body
/// opens with, so it can reach the tables of the database.
///
/// Public because the body in `SpacetimeDSLMethod::method_impl` relies on it without
/// containing it: `spacetimedsl_derive` puts it at the top of every method it renders, and a
/// crate rendering the methods itself has to do the same.
pub fn table_traits_import() -> TokenStream {
    quote! {
        use ::spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};
    }
}
