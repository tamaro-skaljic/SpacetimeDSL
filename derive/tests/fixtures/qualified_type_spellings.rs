//! Covers how a column's type is classified when the user spells it with a qualified path
//! instead of the bare name.
//!
//! `String` columns are passed as `&str` and read by reference, and `Option<T>` columns
//! get the `is_option` handling. Both answers are derived by rendering the type to text and
//! comparing that text, so `std::string::String` and `core::option::Option<T>` -- the same
//! types, differently spelled -- fall through to the default and generate different code
//! from their bare equivalents.
//!
//! Each pair below is one type written two ways. Every pair should generate the same code;
//! today it does not, and this fixture pins that so the fix reads as a diff.

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
