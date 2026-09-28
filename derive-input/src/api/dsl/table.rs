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

/// What `#[spacetimedsl::dsl]` declares for the struct.
#[derive(Clone)]
pub struct SpacetimeDSLTable {
    /// `None` for an ordinary table.
    pub singleton: Option<SingletonKind>,
    /// The `plural_name` of the table, used in the names of the methods which return many
    /// rows, such as `get_all_entities`. For a singleton, which has no `plural_name`, the
    /// accessor of the table.
    pub plural_name: Ident,
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
    /// The `Create<Table>` struct the create method takes. `None` when the table has no create
    /// method.
    pub create_dsl_method_arg: Option<CreateDSLMethodArg>,
    /// The hooks `#[dsl(hook(...))]` declares.
    pub hooks: SpacetimeDSLMethodHooks,
}

impl SpacetimeDSLTable {
    pub fn is_singleton(&self) -> bool {
        self.singleton.is_some()
    }

    pub fn is_soft_deletable(&self) -> bool {
        self.soft_delete_marker.is_some()
    }

    /// Whether `get_<table>` falls back to `DefaultSingleton::get_default` instead of failing.
    pub fn singleton_has_default(&self) -> bool {
        self.singleton == Some(SingletonKind::WithDefault)
    }
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
/// method, `on_soft_deletion` when it is soft-deletable, both when it is both.
#[derive(Clone)]
pub struct OnDeleteStrategiesOfReferencingTables {
    /// `Some` when the table has a delete method.
    pub on_deletion: Option<CascadeEntryPoints>,
    /// `Some` when the table is soft-deletable.
    pub on_soft_deletion: Option<CascadeEntryPoints>,
}

impl OnDeleteStrategiesOfReferencingTables {
    /// The entry points the table holds: those for deletion first, then those for soft
    /// deletion.
    pub fn entry_points(&self) -> impl Iterator<Item = &CascadeEntryPoints> {
        [&self.on_deletion, &self.on_soft_deletion]
            .into_iter()
            .flatten()
    }
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

impl OnDeleteStrategiesOfTheReferencedTable {
    /// The entry points held for the referenced table: those for deletion first, then those
    /// for soft deletion.
    pub fn entry_points(&self) -> impl Iterator<Item = &CascadeEntryPoints> {
        [&self.on_deletion, &self.on_soft_deletion]
            .into_iter()
            .flatten()
    }
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
    /// of this table is removed. `None` when no table references this one.
    pub on_delete_strategies_of_referencing_tables: Option<OnDeleteStrategiesOfReferencingTables>,
    /// The strategy implementations for each table this table references with a foreign key.
    pub on_delete_strategies_of_this_table: Vec<OnDeleteStrategiesOfTheReferencedTable>,
    /// The methods of each multi-column index, in the order of
    /// `SpacetimeDBTable::multi_column_indices`.
    pub multi_column_indices: Vec<SpacetimeDSLColumnMethods>,
    /// Methods this table adds to the wrapper types of its foreign key columns.
    pub wrapper_methods: Vec<WrapperMethod>,
}
