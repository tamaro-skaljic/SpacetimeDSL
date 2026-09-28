//! The analysis SpacetimeDSL makes of a struct carrying `#[spacetimedsl::dsl]` and
//! `#[spacetimedb::table]`, as a data-transfer structure for procedural macro crates.
//!
//! [`api::Table::try_parse`] is the one entry point. What it returns is plain data: every
//! field of every type in [`api`] is public, and every field is part of the semver
//! contract of this crate, so adding, removing or renaming one is a breaking change.
//! A test in the `spacetimedsl_derive` crate destructures the whole structure without
//! `..` to hold that contract.
//!
//! The generated token streams — `method_impl`, `wrapper_impl`, `struct_impl` and the
//! other `TokenStream` fields — are opaque output for a macro to splice into its own
//! expansion, not something to inspect: their content changes with every release that
//! changes what SpacetimeDSL generates.

pub mod api {
    pub mod rust;

    pub mod db;

    pub mod dsl;

    pub mod runtime;

    /// A struct carrying `#[spacetimedb::table]` and `#[spacetimedsl::dsl]`, and its columns.
    #[derive(Clone)]
    pub struct Table {
        /// The struct as Rust sees it.
        pub rust_struct: rust::table::RustStruct,
        /// What `#[spacetimedb::table]` declares for the struct.
        pub spacetimedb_table: db::table::SpacetimeDBTable,
        /// What `#[spacetimedsl::dsl]` declares for the struct.
        pub spacetimedsl_table: dsl::table::SpacetimeDSLTable,
        /// Every field of the struct, in declaration order.
        pub columns: Vec<Column>,
        /// The `#[primary_key]` column, also contained in `columns`. For a singleton it is
        /// the `id: u8` column SpacetimeDSL injects.
        pub primary_key_column: Column,
        /// The DSL methods of the table which belong to no single column.
        pub spacetimedsl_methods: dsl::table::SpacetimeDSLTableMethods,
    }

    impl Table {
        /// Analyses a struct the way `#[spacetimedsl::dsl]` does.
        ///
        /// Call it from your own [attribute macro](https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros)
        /// with the arguments of the `#[dsl(...)]` attribute as `args` and the item it is
        /// applied to as `input`, to build upon a SpacetimeDB Rust server module with
        /// SpacetimeDSL. It fails with a spanned diagnostic for every input
        /// `#[spacetimedsl::dsl]` rejects.
        pub fn try_parse(
            args: proc_macro2::TokenStream,
            input: &syn::DeriveInput,
        ) -> syn::Result<Table> {
            crate::internal::try_parse(args, input)
        }
    }

    /// A field of a struct carrying `#[spacetimedb::table]` and `#[spacetimedsl::dsl]`.
    #[derive(Clone)]
    pub struct Column {
        /// The field as Rust sees it.
        pub rust_field: rust::column::RustField,
        /// What `#[spacetimedb::table]` declares for the field.
        pub spacetimedb_column: db::column::SpacetimeDBColumn,
        /// What `#[spacetimedsl::dsl]` declares for the field.
        pub spacetimedsl_column: dsl::column::SpacetimeDSLColumn,
        /// The DSL methods of the single-column index on the field. `None` when the field
        /// has no single-column index.
        pub spacetimedsl_methods: Option<dsl::column::SpacetimeDSLColumnMethods>,
    }
}

mod internal;
