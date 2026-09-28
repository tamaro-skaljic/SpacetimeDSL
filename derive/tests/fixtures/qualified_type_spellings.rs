//! Covers how a column's type is classified when the user spells it with a qualified path
//! instead of the bare name.
//!
//! `ColumnTypeKind::of` classifies by the path's last segment and accepts a path that is
//! bare or rooted in `std`, `core` or `alloc`, so `std::string::String` classifies as
//! `String` and `core::option::Option<T>` as `Optional`. A primitive may also be spelled
//! `core::primitive::u64` or `std::primitive::u64`, and a SpacetimeDB type
//! `spacetimedb::Timestamp` or `::spacetimedb::Timestamp`.
//!
//! Each pair in `Document` is one type written two ways, and each pair generates the same
//! code - apart from where `prettyplease` wraps a line, because the two spellings are
//! different lengths. `Revision`, `ArchivedNote` and `HiddenNote` spell every column they
//! have a role for qualified: a qualified primitive still counts as an unsigned integer for
//! the reference-integrity guard of its foreign key, and qualified timestamps and flags
//! still fill the timestamp and soft-delete marker roles.

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

#[spacetimedsl::dsl(plural_name = drafts, method(update = false))]
#[spacetimedb::table(accessor = draft, public)]
pub struct Draft {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = revision)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = revisions, method(update = true))]
#[spacetimedb::table(accessor = revision, public)]
pub struct Revision {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(DraftId)]
    #[foreign_key(path = self, table = draft, column = id, on_delete = Delete)]
    pub draft_id: core::primitive::u64,

    #[set_on_create]
    created_at: ::spacetimedb::Timestamp,

    #[set_on_update]
    modified_at: std::option::Option<spacetimedb::Timestamp>,
}

#[spacetimedsl::dsl(
    plural_name = archived_notes,
    method(update = true, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = archived_note, public)]
pub struct ArchivedNote {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub text: String,

    deleted_at: core::option::Option<::spacetimedb::Timestamp>,
}

#[spacetimedsl::dsl(
    plural_name = hidden_notes,
    method(update = true, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = hidden_note, public)]
pub struct HiddenNote {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub text: String,

    #[set_on_soft_delete]
    hidden: core::primitive::bool,
}
