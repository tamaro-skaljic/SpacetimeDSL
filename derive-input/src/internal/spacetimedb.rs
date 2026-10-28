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
    quote::{ToTokens, format_ident, quote},
    syn::{Ident, Path},
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

/// `<#module_path::<accessor>__TableHandle as ::spacetimedb::Table>::Row`: the struct of the
/// table `table_accessor` in `module_path`, named without knowing it. SpacetimeDB generates the
/// handle type `<accessor>__TableHandle` next to every table and implements `Table` for it with
/// the struct as `Row`.
pub fn table_row_type(module_path: &Path, table_accessor: &Ident) -> TokenStream {
    let table_handle = table_handle_type(module_path, table_accessor);

    quote! {
        <#table_handle as ::spacetimedb::Table>::Row
    }
}

/// `<accessor>__TableHandle`, the handle type SpacetimeDB generates next to the table
/// `table_accessor`, with the visibility of its struct. Spanned like `table_accessor`, so that
/// an error about the handle underlines the `table = …` which names it.
pub fn table_handle_name(table_accessor: &Ident) -> Ident {
    format_ident!(
        "{table_accessor}__TableHandle",
        span = table_accessor.span()
    )
}

/// `#module_path::<accessor>__TableHandle`: [`table_handle_name`] in `module_path`.
pub fn table_handle_type(module_path: &Path, table_accessor: &Ident) -> TokenStream {
    let table_handle = table_handle_name(table_accessor);

    quote! {
        #module_path::#table_handle
    }
}
