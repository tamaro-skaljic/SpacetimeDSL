pub mod api {
    pub mod rust;

    pub mod db;

    pub mod dsl;

    /**
     * The representation of a Rust struct with `#[spacetimedb::table]` attribute macro and its columns with optional SpacetimeDSL support.
     */
    #[cfg_attr(feature = "clone", derive(Clone))]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "partial-eq", derive(PartialEq))]
    #[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
    #[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
    pub struct Table {
        pub rust: rust::RustStruct,
        pub spacetimedb: db::SpacetimeDBTable,
        pub spacetimedsl: Option<dsl::table::DSLTable>,
        pub columns: Vec<Column>,
    }

    impl Table {
        /**
         * Supply the &DeriveInput which you've got from your own [derive macro](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)
         * to this function to build upon your SpacetimeDB rust server module with SpacetimeDSL support out-of-the-box in the default feature set.
         */
        pub fn parse(args: syn::Attribute, item: syn::DeriveInput) -> syn::Result<Table> {
            crate::internal::try_parse(args, item)
        }
    }

    /**
     * The representation of a field of a Rust struct with `#[spacetimedb::table]` attribute macro with optional SpacetimeDSL support.
     */
    #[cfg_attr(feature = "clone", derive(Clone))]
    #[cfg_attr(feature = "debug", derive(Debug))]
    #[cfg_attr(feature = "partial-eq", derive(PartialEq))]
    #[cfg_attr(feature = "partial-ord", derive(PartialOrd))]
    #[cfg_attr(feature = "spacetime-type", derive(spacetimedb::SpacetimeType))]
    pub struct Column {
        pub rust: rust::RustField,
        pub spacetimedb: db::DBColumn,
        pub spacetimedsl: Option<dsl::column::DSLColumn>,
    }
}

mod internal;
