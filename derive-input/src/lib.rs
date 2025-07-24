pub mod api {
    pub mod rust;

    pub mod db;

    pub mod dsl;

    /**
     * The representation of a Rust struct with `#[table]` and `#[dsl]` attribute macros and its columns.
     */
    #[derive(Clone)]
    pub struct Table {
        pub rust_struct: rust::table::RustStruct,
        pub spacetimedb_table: db::table::SpacetimeDBTable,
        pub spacetimedsl_table: dsl::table::SpacetimeDSLTable,
        pub columns: Vec<Column>,
        pub primary_key_column: Column,
        pub spacetimedsl_methods: dsl::table::SpacetimeDSLTableMethods,
    }

    impl Table {
        /**
         * Supply the &DeriveInput which you've got from your own [derive macro](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)
         * to this function to build upon your SpacetimeDB rust server module with SpacetimeDSL.
         */
        pub fn try_parse(
            args: proc_macro2::TokenStream,
            input: &syn::DeriveInput,
        ) -> syn::Result<Table> {
            crate::internal::try_parse(args, input)
        }
    }

    /**
     * The representation of a field of a Rust struct with `#[table]` and `#[dsl]` attribute macros.
     */
    #[derive(Clone)]
    pub struct Column {
        pub rust_field: rust::column::RustField,
        pub spacetimedb_column: db::column::SpacetimeDBColumn,
        pub spacetimedsl_column: dsl::column::SpacetimeDSLColumn,
        pub spacetimedsl_methods: Option<dsl::column::SpacetimeDSLColumnMethods>,
    }
}

#[doc(hidden)]
mod internal;
