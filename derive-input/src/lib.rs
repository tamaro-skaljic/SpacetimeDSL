pub mod api {
    pub mod rust;

    pub mod db;

    pub mod dsl;

    /**
     * The representation of a Rust struct with `#[spacetimedb::table]` and `#[spacetimedsl::table]` attribute macros and its columns.
     */
    #[cfg_attr(feature = "clone", derive(Clone))]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "partial-eq", derive(PartialEq))]
    #[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
    #[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
    pub struct Table {
        pub rust_struct: rust::RustStruct,
        pub spacetimedb_table: db::SpacetimeDBTable,
        pub spacetimedsl_table: dsl::table::SpacetimeDSLTable,
        pub columns: Vec<Column>,
        pub spacetimedsl_methods: dsl::method::SpacetimeDSLTableMethods,
    }

    impl Table {
        /**
         * Supply the &DeriveInput which you've got from your own [derive macro](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)
         * to this function to build upon your SpacetimeDB rust server module with SpacetimeDSL.
         */
        pub fn try_parse(item: &syn::DeriveInput) -> syn::Result<Table> {
            crate::internal::try_parse(item)
        }
    }

    /**
     * The representation of a field of a Rust struct with `#[spacetimedb::table]` and `#[spacetimedsl::table]` attribute macros.
     */
    #[cfg_attr(feature = "clone", derive(Clone))]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "partial-eq", derive(PartialEq))]
    #[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
    #[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
    pub struct Column {
        pub rust_field: rust::RustField,
        pub spacetimedb_column: db::SpacetimeDBColumn,
        pub spacetimedsl_column: dsl::column::SpacetimeDSLColumn,
        pub spacetimedsl_methods: Option<dsl::method::SpacetimeDSLColumnMethods>,
    }
}

#[doc(hidden)]
mod internal;
