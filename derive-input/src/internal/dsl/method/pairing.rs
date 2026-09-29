//! The compile-time checks which pair a table with the tables on the other side of its foreign
//! keys.
//!
//! A table cannot see how the tables on the other side are declared, only what their expansion
//! emits. So each side declares marker traits for what it offers and imports from the other
//! side the traits it needs. A counterpart which does not offer one is an unresolved import,
//! and the trait's name says what to change.
//!
//! These names are part of the generated API: both sides build them here, from the same two
//! table names, and changing one breaks every module generated against the previous name
//! until it is regenerated.

use {
    super::{
        context::{MethodGenerationContext, TableContributions},
        foreign_key::foreign_key_of,
        removal::Removal,
    },
    crate::api::Column,
    quote::format_ident,
    std::collections::BTreeMap,
    syn::{Ident, Path, PathSegment},
};

/// The traits the table of `context` declares and imports, as the referenced table its
/// `#[referenced_by]` attributes describe, and as the referencing table its foreign keys make
/// it.
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

/// For every table the `#[referenced_by]` attributes name and every removal this table
/// performs: the trait which says so, and the import of the trait the other table declares
/// when a foreign key of it declares a strategy for that removal.
fn as_referenced_table(context: &MethodGenerationContext) -> TableContributions {
    let MethodGenerationContext {
        spacetimedsl_table,
        singular_table_name,
        ..
    } = context;

    let mut contributions = TableContributions::default();

    for referencing_table in &spacetimedsl_table.referencing_tables {
        for removal in Removal::ALL {
            if !removal.is_performed_by(spacetimedsl_table) {
                continue;
            }

            contributions
                .compile_error_checks
                .insert(referenced_table_compile_error_check(
                    removal,
                    singular_table_name,
                    &referencing_table.table_name,
                ));

            contributions
                .compile_error_check_imports
                .push(imported_from(
                    &referencing_table.path,
                    referencing_table_compile_error_check(
                        removal,
                        &referencing_table.table_name,
                        singular_table_name,
                    ),
                ));
        }
    }

    contributions
}

/// For every table the foreign keys reference and every removal one of them declares a
/// strategy for: the trait which says so, and the import of the trait the referenced table
/// declares when it performs that removal.
fn as_referencing_table(
    singular_table_name: &Ident,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let mut contributions = TableContributions::default();

    for (referenced_table_name, columns_with_foreign_key) in foreign_key_columns_by_referenced_table
    {
        let referenced_table_path = &foreign_key_of(columns_with_foreign_key[0]).path;

        for removal in Removal::ALL {
            let declares_a_strategy = columns_with_foreign_key.iter().any(|column| {
                removal
                    .strategy_declared_by(foreign_key_of(column))
                    .is_some()
            });

            if !declares_a_strategy {
                continue;
            }

            contributions
                .compile_error_checks
                .insert(referencing_table_compile_error_check(
                    removal,
                    singular_table_name,
                    referenced_table_name,
                ));

            contributions
                .compile_error_check_imports
                .push(imported_from(
                    referenced_table_path,
                    referenced_table_compile_error_check(
                        removal,
                        referenced_table_name,
                        singular_table_name,
                    ),
                ));
        }
    }

    contributions
}

/// `this_compilation_error_occurs_because_the_<referenced>_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_<referencing>_table`,
/// or `…_soft_deletable…` for a soft deletion.
fn referenced_table_compile_error_check(
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

/// `this_compilation_error_occurs_because_the_<referencing>_table_has_no_foreign_key_attribute_with_on_delete_defined_referencing_the_<referenced>_table`,
/// or `…on_soft_delete…` for a soft deletion.
fn referencing_table_compile_error_check(
    removal: Removal,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    let strategy_argument = removal.strategy_argument();

    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_with_{strategy_argument}_defined_referencing_the_{referenced_table_name}_table"
    )
}

/// `<module_path>::<trait_name>`: where the expansion imports another table's trait from.
fn imported_from(module_path: &Path, trait_name: Ident) -> Path {
    let mut path = module_path.clone();
    path.segments.push(PathSegment::from(trait_name));
    path
}
