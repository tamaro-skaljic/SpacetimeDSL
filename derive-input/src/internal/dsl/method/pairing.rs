//! The compile-time checks which pair a table with the tables on the other side of its foreign
//! keys.
//!
//! A table cannot see how the tables on the other side are declared, only what their expansion
//! emits. So each side declares marker traits for what it offers and imports from the other
//! side the traits it needs. A counterpart which does not offer one is an unresolved import,
//! and the trait's name says what to change.
//!
//! These names are part of the generated API: both sides build them here, from the same two
//! table names.

use {
    super::{
        context::{MethodGenerationContext, TableContributions},
        foreign_key::{first_column_of, foreign_key_of},
        removal::Removal,
    },
    crate::api::Column,
    quote::format_ident,
    std::collections::BTreeMap,
    syn::{Ident, Path, PathSegment},
};

/// The traits the table of `context` declares and imports, as the table its
/// `#[referenced_by]` attributes describe and as the table whose foreign keys reference others.
pub fn for_table(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let mut contributions = as_referenced_table(context);

    contributions.merge(as_referencing_table(
        &context.singular_table_name,
        foreign_key_columns_by_referenced_table,
    ));

    contributions
}

/// For every table the `#[referenced_by]` attributes name: per removal, the trait which says
/// whether this table performs it, and the import of the trait which says that the other
/// table has a foreign key back.
fn as_referenced_table(context: &MethodGenerationContext) -> TableContributions {
    let MethodGenerationContext {
        spacetimedsl_table,
        singular_table_name,
        ..
    } = context;

    let mut contributions = TableContributions::default();

    for referencing_table in &spacetimedsl_table.referencing_tables {
        let referencing_table_name = &referencing_table.table_name;

        for removal in Removal::ALL {
            let declared = match removal.is_performed_by(spacetimedsl_table) {
                true => removal_is_possible(removal, singular_table_name, referencing_table_name),
                false => {
                    removal_is_impossible(removal, singular_table_name, referencing_table_name)
                }
            };

            contributions.compile_error_checks.insert(declared);
        }

        contributions
            .compile_error_check_imports
            .push(imported_from(
                &referencing_table.path,
                foreign_key_exists(referencing_table_name, singular_table_name),
            ));
    }

    contributions
}

/// For every table the foreign keys reference: the trait which says that this table has a
/// foreign key to it, and per removal the import of the trait matching each foreign key.
///
/// A foreign key declares a strategy exactly when the referenced table performs the removal,
/// so a group where one foreign key declares it and another does not imports both traits,
/// and one of them is missing.
fn as_referencing_table(
    singular_table_name: &Ident,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let mut contributions = TableContributions::default();

    for (referenced_table_name, columns_with_foreign_key) in foreign_key_columns_by_referenced_table
    {
        let referenced_table_path = &foreign_key_of(first_column_of(columns_with_foreign_key)).path;

        contributions
            .compile_error_checks
            .insert(foreign_key_exists(
                singular_table_name,
                referenced_table_name,
            ));

        for removal in Removal::ALL {
            let declares_a_strategy = |column: &&Column| {
                removal
                    .strategy_declared_by(foreign_key_of(column))
                    .is_some()
            };

            if columns_with_foreign_key.iter().any(declares_a_strategy) {
                contributions
                    .compile_error_check_imports
                    .push(imported_from(
                        referenced_table_path,
                        removal_is_possible(removal, referenced_table_name, singular_table_name),
                    ));
            }

            if !columns_with_foreign_key.iter().all(declares_a_strategy) {
                contributions
                    .compile_error_check_imports
                    .push(imported_from(
                        referenced_table_path,
                        removal_is_impossible(removal, referenced_table_name, singular_table_name),
                    ));
            }
        }
    }

    contributions
}

/// Declared by a referenced table for each table its `#[referenced_by]` names and each
/// removal it performs. A foreign key declaring a strategy for that removal imports it.
fn removal_is_possible(
    removal: Removal,
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    let capability = match removal {
        Removal::Hard => "deletable",
        Removal::Soft => "soft_deletable",
    };

    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_{capability}_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

/// Declared by a referenced table for each table its `#[referenced_by]` names and each
/// removal it does not perform. A foreign key declaring no strategy for that removal imports
/// it.
fn removal_is_impossible(
    removal: Removal,
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    let strategy_argument = removal.strategy_argument();

    format_ident!(
        "this_compilation_error_occurs_because_your_foreign_key_referencing_the_{referenced_table_name}_table_needs_to_define_a_strategy_for_{strategy_argument}_or_the_{referenced_table_name}_table_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

/// Declared by a referencing table for each table its foreign keys reference. The referenced
/// table imports it for each table its `#[referenced_by]` names.
fn foreign_key_exists(referencing_table_name: &Ident, referenced_table_name: &Ident) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_referencing_the_{referenced_table_name}_table"
    )
}

/// `<module_path>::<trait_name>`: where the expansion imports another table's trait from.
fn imported_from(module_path: &Path, trait_name: Ident) -> Path {
    let mut path = module_path.clone();
    path.segments.push(PathSegment::from(trait_name));
    path
}
