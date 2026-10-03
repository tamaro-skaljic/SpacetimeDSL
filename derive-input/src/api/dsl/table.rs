use {
    super::reference::ReferencingTable,
    crate::api::dsl::{
        column::SpacetimeDSLColumnMethods,
        hook::SpacetimeDSLMethodHooks,
        method::{SpacetimeDSLArg, SpacetimeDSLMethod},
        soft_delete::SoftDeleteMarker,
        wrapper::WrapperMethod,
    },
    proc_macro2::TokenStream,
    std::collections::BTreeSet,
    syn::Ident,
};

/// How many rows a singleton table holds, which decides which methods it earns.
///
/// `WithoutDefault` is `#[dsl(singleton)]`: at most one row, and `get_<table>` fails while
/// the row is absent. `WithDefault` is `#[dsl(singleton(with_default))]`: exactly one row
/// from the caller's point of view, because `get_<table>` falls back to the default the
/// table's struct supplies through the `DefaultSingleton` trait.
#[derive(Clone, Copy, PartialEq)]
pub enum SingletonKind {
    WithoutDefault,
    WithDefault,
}

/// Whether `#[spacetimedsl::dsl]` declares an ordinary table or a singleton.
#[derive(Clone)]
pub enum SpacetimeDSLTableKind {
    /// A table without `singleton`, which holds any number of rows.
    Normal {
        /// `plural_name = ...`, used in the names of the methods which return many rows,
        /// such as `get_all_entities`.
        plural_name: Ident,
    },
    /// `#[dsl(singleton)]` or `#[dsl(singleton(with_default))]`. A singleton has no
    /// `plural_name`: it holds one row, so no method returns many.
    Singleton(SingletonKind),
}

/// What `#[spacetimedsl::dsl]` declares for the struct.
#[derive(Clone)]
pub struct SpacetimeDSLTable {
    /// Whether the table is an ordinary table, with its `plural_name`, or a singleton.
    pub kind: SpacetimeDSLTableKind,
    /// `method(update = ...)`, which every `#[dsl]` states.
    pub has_update_method: bool,
    /// `method(delete = ...)`, `true` unless stated otherwise.
    pub has_delete_method: bool,
    /// The column a soft deletion writes, if the table is soft-deletable.
    ///
    /// `Some` and `#[dsl(method(soft_delete = true))]` imply each other: the parser rejects
    /// either one without the other, so this is the single question every generator asks.
    pub soft_delete_marker: Option<SoftDeleteMarker>,
    /// The column the create method sets to the current timestamp: `#[set_on_create]`, or
    /// one of the conventional names `created_at` and `inserted_at`. `None` when there is none.
    pub on_insert_set_current_timestamp_column_name: Option<Ident>,
    /// The column the update method sets to the current timestamp: `#[set_on_update]`, or
    /// one of the conventional names `modified_at` and `updated_at`. `None` when there is none.
    pub on_update_set_current_timestamp_column_name: Option<Ident>,
    /// The tables `#[referenced_by(...)]` on the primary key column names.
    pub referencing_tables: Vec<ReferencingTable>,
    /// The names of the marker traits the expansion declares for the tables on the other side
    /// of a foreign key. A table whose counterpart does not declare the trait it expects fails
    /// to compile with an error naming the missing configuration.
    pub compile_error_checks: BTreeSet<Ident>,
    /// The marker traits the tables on the other side of a foreign key have to declare, as the
    /// paths the expansion imports them from. A missing one is an unresolved import whose name
    /// says what to change.
    pub compile_error_check_imports: Vec<syn::Path>,
    /// The `Create<Table>` struct the create method takes. `None` when the table has no create
    /// method.
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    /// The hooks `#[dsl(hook(...))]` declares.
    pub hooks: SpacetimeDSLMethodHooks,
    /// The documentation `#[spacetimedsl::dsl]` appends to the struct's own: the table's
    /// foreign keys and the tables which reference it. Empty when it has neither.
    pub struct_doc_comment: String,
}

/// The `Create<Table>` struct the create method takes, with one field per column the caller
/// supplies.
#[derive(Clone)]
pub struct CreateDSLMethodArg {
    /// `Create<Table>`, such as `CreateEntity`.
    pub struct_name: Ident,
    /// The fields of the struct.
    pub struct_members: Vec<SpacetimeDSLArg>,
    /// The definition of the struct.
    pub struct_impl: TokenStream,
}

/// The two entry points one kind of removal earns.
///
/// Both exist or neither does: a removal reaches a table either one row at a time or many
/// at once, and a foreign key on the other side does not know which method will reach it.
#[derive(Clone)]
pub struct CascadeEntryPoints {
    /// Called after one row was removed.
    pub after_one_row: SpacetimeDSLMethod,
    /// Called after many rows were removed at once.
    pub after_multiple_rows: SpacetimeDSLMethod,
}

/// The cascade entry points a table earns when another table references it.
///
/// One pair per kind of removal the table can perform: `on_deletion` when it has a delete
/// method, `on_soft_deletion` when it is soft-deletable, both when it is both. At least one of
/// the two is `Some`.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfReferencingTables {
    /// `Some` when the table has a delete method.
    pub on_deletion: Option<CascadeEntryPoints>,
    /// `Some` when the table is soft-deletable.
    pub on_soft_deletion: Option<CascadeEntryPoints>,
}

/// The strategy implementations a table earns for one table it references.
///
/// One of these per referenced table, which is why `SpacetimeDSLTableMethods` holds a
/// `Vec` of them rather than parallel `Vec`s that could go out of step. Inside, one pair
/// per kind of removal this table's foreign keys to that table declare a strategy for.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfTheReferencedTable {
    /// `Some` when the foreign keys declare an `on_delete` strategy.
    pub on_deletion: Option<CascadeEntryPoints>,
    /// `Some` when the foreign keys declare an `on_soft_delete` strategy.
    pub on_soft_deletion: Option<CascadeEntryPoints>,
}

/// The DSL methods of a table which belong to no single column.
#[derive(Clone)]
pub struct SpacetimeDSLTableMethods {
    /// `None` on a `SingletonKind::WithDefault` table: its row is written by
    /// `upsert_<table>`, so it has no create method and no `Create<Table>` argument struct.
    pub create: Option<SpacetimeDSLMethod>,
    /// `get_all_<plural_name>`. `None` on a singleton.
    pub get_all: Option<SpacetimeDSLMethod>,
    /// `count_of_all_<plural_name>`. `None` on a singleton.
    pub get_count: Option<SpacetimeDSLMethod>,
    /// The cascade entry points the tables named in `#[referenced_by(...)]` call when a row
    /// of this table is removed. `None` when no table references this one, or when this table
    /// neither deletes nor soft-deletes rows.
    pub on_delete_strategies_of_referencing_tables: Option<OnDeleteStrategiesOfReferencingTables>,
    /// The strategy implementations for each table this table references with a foreign key
    /// which declares a strategy.
    pub on_delete_strategies_of_this_table: Vec<OnDeleteStrategiesOfTheReferencedTable>,
    /// The methods of each multi-column index, in the order of
    /// `SpacetimeDBTable::multi_column_indices`.
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
    /// Methods this table adds to the wrapper types of its foreign key columns.
    pub wrapper_methods: Vec<WrapperMethod>,
    /// Methods this table adds to the wrapper type of its primary key, one per foreign key
    /// column besides the primary key, which look up the row that column of the row with a key
    /// references, such as `server_id.get_season(&dsl)`.
    ///
    /// Every table of a struct shares the struct's wrapper types, so the key names a row in
    /// each of them and the lookup would be ambiguous: `spacetimedsl_derive` emits these methods
    /// only for a struct with a single `#[dsl]` attribute. Empty for a singleton, whose
    /// injected primary key has no wrapper type.
    pub referenced_row_methods: Vec<WrapperMethod>,
}
