//! The rustdoc text which says how a table relates to other tables through `#[foreign_key]`
//! and `#[referenced_by]`.
//!
//! The write and removal methods, the accessors of a foreign key column and the struct show
//! the same relationships, so each fact is phrased once here.

use {
    super::{reference_integrity, removal::Removal},
    crate::{api::dsl::reference::ReferencingTable, internal::column::InternalColumn},
    quote::ToTokens,
    syn::{Ident, Path},
};

/// `doc`, followed by `section` as a paragraph of its own, or `doc` alone when `section` is
/// empty.
pub fn with_section(doc: String, section: String) -> String {
    match section.is_empty() {
        true => doc,
        false => format!("{doc}\n\n{section}"),
    }
}

/// The `# Foreign keys` section of a method which checks, before it writes, that the foreign
/// keys among `checked_columns` reference a row. Empty when none of them has a foreign key.
pub fn reference_checks(lead: &str, checked_columns: &[&InternalColumn]) -> String {
    let bullets: Vec<String> = checked_columns
        .iter()
        .filter_map(|column| {
            let foreign_key = column.spacetimedsl_column_foreign_key.as_ref()?;
            let value_referencing_no_row =
                reference_integrity::documented_value_referencing_no_row(
                    column.rust_field_type_kind,
                )
                .map(|value| format!(", or {value}"))
                .unwrap_or_default();

            Some(format!(
                "- `{}`: a row of {}{value_referencing_no_row}",
                column.rust_field_name,
                table_and_module(&foreign_key.table_name, &foreign_key.path),
            ))
        })
        .collect();

    section("Foreign keys", Some(lead), &bullets)
}

/// The `# Cascade` section of a method which removes rows other tables reference: the tables
/// whose strategies for `removal` it runs.
pub fn cascade(removal: Removal, referencing_tables: &[ReferencingTable]) -> String {
    let bullets: Vec<String> = referencing_tables
        .iter()
        .map(|referencing_table| {
            format!(
                "- {}",
                table_and_module(&referencing_table.table_name, &referencing_table.path)
            )
        })
        .collect();

    let lead = format!(
        "Runs the `{}` strategies which the foreign keys of these tables declare:",
        removal.strategy_argument()
    );

    section("Cascade", Some(lead.as_str()), &bullets)
}

/// A doc comment section: its heading, a lead sentence when there is one, and its bullets.
/// Nothing when there are no bullets.
fn section(heading: &str, lead: Option<&str>, bullets: &[String]) -> String {
    if bullets.is_empty() {
        return String::new();
    }

    let lead = lead.map(|lead| format!("{lead}\n\n")).unwrap_or_default();

    format!("# {heading}\n\n{lead}{}", bullets.join("\n"))
}

/// "the `warehouse` table (`self`)": a table and the module path its attribute names it by,
/// written the way the user wrote it.
fn table_and_module(table_name: &Ident, module_path: &Path) -> String {
    let module_path = module_path.to_token_stream().to_string().replace(' ', "");

    format!("the `{table_name}` table (`{module_path}`)")
}
