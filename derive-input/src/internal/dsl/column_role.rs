//! The column names which claim a role without an attribute. A column named `created_at`
//! is set on create as if it carried `#[set_on_create]`, and so on for every role.
//!
//! Every role is looked up the same way, through [`claims`], and every diagnostic that
//! lists the names renders them from these constants.

pub const SET_ON_CREATE_COLUMN_NAMES: [&str; 2] = ["created_at", "inserted_at"];
pub const SET_ON_UPDATE_COLUMN_NAMES: [&str; 2] = ["modified_at", "updated_at"];
pub const SOFT_DELETE_FLAG_COLUMN_NAMES: [&str; 2] = ["deleted", "removed"];
pub const SOFT_DELETE_TIMESTAMP_COLUMN_NAMES: [&str; 2] = ["deleted_at", "removed_at"];

/// Whether `column_name` claims the role `names` belong to.
pub fn claims(names: &[&str], column_name: &str) -> bool {
    names.contains(&column_name)
}

/// `` `a`/`b` ``, the way a diagnostic names the alternatives in a sentence about one column.
pub fn slash_separated(names: &[&str]) -> String {
    quoted(names).join("/")
}

/// `` `a` or `b` ``, the way a diagnostic offers the alternatives to choose from.
pub fn or_separated(names: &[&str]) -> String {
    quoted(names).join(" or ")
}

fn quoted(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| format!("`{name}`")).collect()
}
