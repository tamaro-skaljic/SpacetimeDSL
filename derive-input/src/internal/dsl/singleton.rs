//! The primary key a singleton table carries.
//!
//! A singleton table has no primary key of its own, so the `derive` crate injects one:
//! `inject_singleton_primary_key` in `derive/src/lib.rs` adds `#[primary_key] id: u8` as
//! the struct's first field. `derive-input` never sees that injection happen, only the
//! field it leaves behind, so everything it generates for a singleton has to name the
//! field, its type and its single legal value by hand. This module is where it names
//! them, so a change to the injected key has one place to reach in each crate.

use proc_macro2::{Literal, Span};
use quote::ToTokens;
use syn::{Ident, Path};

/// The name of the injected column.
pub(in crate::internal) const PRIMARY_KEY_NAME: &str = "id";

/// The type of the injected column, spelled the way a `Path` renders it.
const PRIMARY_KEY_TYPE: &str = "u8";

/// The only value the injected column ever holds: a singleton has one row.
const PRIMARY_KEY_VALUE: u8 = 0;

/// The injected column, to call its index accessor or to assign to.
pub(in crate::internal) fn primary_key_ident() -> Ident {
    Ident::new(PRIMARY_KEY_NAME, Span::call_site())
}

/// The injected column's value, as generated code writes it.
pub(in crate::internal) fn primary_key_value() -> Literal {
    Literal::u8_suffixed(PRIMARY_KEY_VALUE)
}

/// The injected column's value, as an error message renders it.
pub(in crate::internal) fn rendered_primary_key_value() -> String {
    PRIMARY_KEY_VALUE.to_string()
}

/// The injected column and its value, as a not-found error renders them: `{ id : 0 }`.
///
/// The surrounding braces and the spacing match what
/// `internal::dsl::method::index::column_names_and_row_values` produces for any other index, so
/// a singleton's not-found message reads like every other table's.
pub(in crate::internal) fn rendered_primary_key() -> String {
    format!("{{ {PRIMARY_KEY_NAME} : {PRIMARY_KEY_VALUE} }}")
}

/// Whether this column is the injected primary key rather than one the user wrote.
///
/// The caller has already established that the table is a singleton; this answers which
/// of its columns the injection added.
pub(in crate::internal) fn is_primary_key_column(name: &Ident, type_name_or_path: &Path) -> bool {
    name == PRIMARY_KEY_NAME && type_name_or_path.to_token_stream().to_string() == PRIMARY_KEY_TYPE
}
