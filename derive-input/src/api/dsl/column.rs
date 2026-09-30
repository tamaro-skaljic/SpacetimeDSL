use {
    super::{
        auto_gen::UUIDVersion, disallow::Disallowed, foreign_key::ForeignKey, getter::Getter,
        setter::Setter, wrapper::WrapperType,
    },
    crate::api::dsl::{method::SpacetimeDSLMethod, mut_getter::MutGetter},
    std::collections::BTreeSet,
};

/// What `#[spacetimedsl::dsl]` declares for one field of the struct.
#[derive(Clone)]
pub struct SpacetimeDSLColumn {
    /// Whether the type of the field is an `Option`.
    pub is_option: bool,
    /// `Some` when the field has `#[create_wrapper]` or `#[use_wrapper(...)]`.
    pub wrapper_type: Option<WrapperType>,
    /// `Some` when the field has `#[foreign_key(...)]`.
    pub foreign_key: Option<ForeignKey>,
    /// `Some` when the field has `#[auto_gen(v4)]` or `#[auto_gen(v7)]`.
    pub auto_generated_uuid_version: Option<UUIDVersion>,
    /// `Some` when the field has `#[creation_default(...)]`: the expression `create_<table>`
    /// fills the column with instead of asking the caller for it.
    pub creation_default: Option<syn::Expr>,
    /// The rules `#[disallow(...)]` states for the value of the field. Empty without the
    /// attribute.
    pub disallowed: BTreeSet<Disallowed>,
    /// `Some` for every field except the primary key SpacetimeDSL injects into a singleton.
    pub getter: Option<Getter>,
    /// `Some` when the field is not private and has no wrapper type.
    pub mut_getter: Option<MutGetter>,
    /// `Some` when the field is not private.
    pub setter: Option<Setter>,
}

/// The DSL methods of one index.
#[derive(Clone)]
pub enum SpacetimeDSLColumnMethods {
    /// The methods of an index no two rows share a value of, which find at most one row.
    ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex),
    /// The methods of an index several rows can share a value of.
    ForIndex(SpacetimeDSLColumnMethodsForIndex),
}

/// The DSL methods of an index no two rows share a value of.
#[derive(Clone)]
pub struct SpacetimeDSLColumnMethodsForUniqueIndex {
    /// `get_<table>_by_<index>`, which fails when no row has the value. For a singleton,
    /// `get_<table>`.
    pub get_one_option: SpacetimeDSLMethod,
    /// `Some` when the table has an update method and this index is the primary key. For a
    /// singleton with a default, the upsert which writes its row.
    pub update: Option<SpacetimeDSLMethod>,
    /// `Some` when the table has a delete method.
    pub delete_one: Option<SpacetimeDSLMethod>,
    /// `Some` when the table is soft-deletable.
    pub soft_delete_one: Option<SpacetimeDSLMethod>,
}

/// The DSL methods of an index several rows can share a value of.
#[derive(Clone)]
pub struct SpacetimeDSLColumnMethodsForIndex {
    /// `get_<plural_name>_by_<index>`, which returns every row with the value.
    pub get_many: SpacetimeDSLMethod,
    /// `Some` when the table has a delete method.
    pub delete_many: Option<SpacetimeDSLMethod>,
    /// `Some` when the table is soft-deletable.
    pub soft_delete_many: Option<SpacetimeDSLMethod>,
}
