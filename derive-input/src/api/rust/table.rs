use {crate::api::rust::visibility::RustVisibility, syn::Ident};

/// The struct carrying `#[spacetimedb::table]`, as Rust sees it.
#[derive(Clone)]
pub struct RustStruct {
    /// The visibility the struct is declared with.
    pub visibility: RustVisibility,
    /// The name of the struct, such as `Entity`.
    pub name: Ident,
}
