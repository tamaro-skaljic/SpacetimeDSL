//! Covers how a column's type is classified when the user spells it with a qualified path
//! instead of the bare name.
//!
//! `ColumnTypeKind::of` classifies by the path's last segment and accepts a path that is
//! bare or rooted in `std`, `core` or `alloc`, so `std::string::String` classifies as
//! `String` and `core::option::Option<T>` as `Optional`.
//!
//! Each pair below is one type written two ways, and each pair generates the same code -
//! apart from where `prettyplease` wraps a line, because the two spellings are different
//! lengths.

#[spacetimedsl::dsl(plural_name = documents, method(update = true))]
#[spacetimedb::table(accessor = document, public)]
pub struct Document {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    pub bare_title: String,

    #[index(btree)]
    pub qualified_title: std::string::String,

    #[index(btree)]
    #[use_wrapper(DocumentBareNote)]
    pub bare_note: Option<u64>,

    #[index(btree)]
    #[use_wrapper(DocumentQualifiedNote)]
    pub qualified_note: core::option::Option<u64>,
}
