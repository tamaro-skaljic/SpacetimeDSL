//! Covers `#[create_wrapper]` without an explicit name, which derives the wrapper type
//! name from the struct and the column. One plain column and one indexed column, because
//! the wrapper reaches the generated methods only through the indexed one.

#[spacetimedsl::dsl(plural_name = labels, method(update = true))]
#[spacetimedb::table(accessor = label, public)]
pub struct Label {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[create_wrapper]
    pub caption: String,

    #[unique]
    #[create_wrapper]
    pub code: u64,
}
