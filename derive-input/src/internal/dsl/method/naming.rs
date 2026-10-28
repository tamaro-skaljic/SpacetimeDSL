//! The identifiers that generated code and the code calling it agree on by name: the
//! accessors of a column, which user code calls and other generated methods call too, the
//! trait a hook function implements, and the identifiers two tables in a foreign key
//! relationship share.
//!
//! A table reaches a table on the other side of a foreign key through a type it does not see
//! the definition of, so both sides have to build the same identifier from the same table
//! name. Both sides build it here. The marker traits which pair the two sides are named in
//! [`super::pairing`].
//!
//! These names are part of the generated API.

use {
    super::removal::Removal, crate::internal::dsl::one_or_multiple::OneOrMultiple,
    ident_case::RenameRule, quote::format_ident, syn::Ident,
};

/// `get_<table>_by_<index>`, the method that finds rows by an index.
pub fn get_by_index_method_name(table_name: &Ident, index_name: &Ident) -> Ident {
    format_ident!("get_{table_name}_by_{index_name}")
}

/// `<Table>Table`, the table type through which the tables on the other side of its foreign
/// keys reach a table: the PascalCase form of `singular_table_name` followed by `Table`, such
/// as `WarehouseTable` for `warehouse`.
///
/// A table emits it, and a table on the other side of a foreign key names it from the
/// `table = …` of its `#[foreign_key]` or `#[referenced_by]`, so both have to call this one
/// function. It is spanned like `singular_table_name`, so that an error about the table type,
/// such as the privacy of a struct in another module, underlines that `table = …`.
pub fn table_type_name(singular_table_name: &Ident) -> Ident {
    format_ident!(
        "{}Table",
        RenameRule::PascalCase.apply_to_field(singular_table_name.to_string()),
        span = singular_table_name.span()
    )
}

/// `<Referencing>TableReferencesThe<Referenced>Table`, the check a referenced table makes that a
/// table its `#[referenced_by]` names has a foreign key back. The check also carries the
/// cascades of `internal::References`, through which the referenced table reaches that table.
pub fn foreign_key_back_check_name(
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    format_ident!(
        "{}ReferencesThe{}",
        table_type_name(referencing_table_name),
        table_type_name(referenced_table_name)
    )
}

/// `get_by_primary_key`, the lookup a table type offers the tables whose foreign keys reference
/// its table.
pub fn get_by_primary_key_method_name() -> Ident {
    format_ident!("get_by_primary_key")
}

/// The method of `internal::References` which runs the strategies of a referencing table after
/// rows of the referenced table were removed, such as `after_one_row_was_deleted`.
pub fn cascade_method_name(removal: Removal, one_or_multiple: &OneOrMultiple) -> Ident {
    let count = one_row_or_multiple_rows(one_or_multiple);
    let suffix = removal_suffix(removal, one_or_multiple);

    format_ident!("after_{count}_{suffix}")
}

/// `get_<column>`, the getter of a column.
pub fn getter_name(column_name: &Ident) -> Ident {
    format_ident!("get_{column_name}")
}

/// `get_<column>_mut`, the mut getter of a column.
pub fn mut_getter_name(column_name: &Ident) -> Ident {
    format_ident!("get_{column_name}_mut")
}

/// `set_<column>`, the setter of a column.
pub fn setter_name(column_name: &Ident) -> Ident {
    format_ident!("set_{column_name}")
}

/// `Create<Table>`, the struct `create_<table>` takes and the `before_insert` hook of the
/// table receives and returns.
pub fn create_request_struct_name(singular_table_name: &Ident) -> Ident {
    format_ident!(
        "Create{}",
        RenameRule::PascalCase.apply_to_field(singular_table_name.to_string())
    )
}

/// The trait a hook function implements: the PascalCase form of `hook_function_name`
/// followed by `Hook`, such as `BeforeEntityInsertHook` for `before_entity_insert`.
///
/// The generator declares the trait under this name and `#[spacetimedsl::hook]` implements
/// it under this name, so both have to call this one function.
pub fn hook_trait_name(hook_function_name: &Ident) -> Ident {
    format_ident!(
        "{}Hook",
        RenameRule::PascalCase.apply_to_field(hook_function_name.to_string())
    )
}

/// How a dispatcher name ends, which is the only part the kind of removal changes.
fn removal_suffix(removal: Removal, one_or_multiple: &OneOrMultiple) -> String {
    removal.past_tense(one_or_multiple).replace([' ', '-'], "_")
}

/// How the beginning of a dispatcher name reads, which is the only part the count changes.
fn one_row_or_multiple_rows(one_or_multiple: &OneOrMultiple) -> &'static str {
    match one_or_multiple {
        OneOrMultiple::One => "one_row",
        OneOrMultiple::Multiple => "multiple_rows",
    }
}

pub fn referenced_table_function_name(
    removal: Removal,
    one_or_multiple: &OneOrMultiple,
    referenced_table_name: &Ident,
) -> Ident {
    let count = one_row_or_multiple_rows(one_or_multiple);
    let suffix = removal_suffix(removal, one_or_multiple);

    format_ident!(
        "execute_on_delete_strategies_of_referencing_tables_after_{count}_of_the_{referenced_table_name}_table_{suffix}"
    )
}

/// The bindings the fragments of a cascade dispatcher share.
///
/// `for_foreign_key` declares the dispatcher's arguments and state, and the strategy
/// fragments `on_delete_strategy.rs` generates read and write them, as do the fragments of
/// one strategy among each other. Nothing checks that the names meet until a user's crate
/// expands the macro, so each is spelled only here.
pub mod cascade_binding {
    use {
        proc_macro2::Span,
        quote::format_ident,
        syn::{Ident, Lifetime},
    };

    /// The DSL the dispatcher receives.
    pub fn dsl() -> Ident {
        format_ident!("dsl")
    }

    /// The entries the dispatcher returns, a `Vec` for one row or a `HashMap` for many.
    pub fn entries() -> Ident {
        format_ident!("entries")
    }

    /// Whether a strategy failed, which ends the labelled block early.
    pub fn error() -> Ident {
        format_ident!("error")
    }

    /// The error which stopped the cascade, if one did: one a hook raised, or a broken
    /// `#[disallow]` rule of a row the cascade wrote.
    pub fn error_from_hook() -> Ident {
        format_ident!("error_from_hook")
    }

    /// The label of the block a failing strategy breaks out of.
    pub fn outer() -> Lifetime {
        Lifetime::new("'outer", Span::call_site())
    }

    /// The primary key value of the one referenced row that was removed.
    pub fn primary_key_value_of_a_row_of_another_table_to_delete() -> Ident {
        format_ident!("primary_key_value_of_a_row_of_another_table_to_delete")
    }

    /// The primary key values of the referenced rows that were removed.
    pub fn primary_key_values_of_rows_of_another_table_to_delete() -> Ident {
        format_ident!("primary_key_values_of_rows_of_another_table_to_delete")
    }

    /// The primary key values of this table's rows a strategy removes in turn.
    pub fn primary_key_values_of_rows_to_delete() -> Ident {
        format_ident!("primary_key_values_of_rows_to_delete")
    }

    /// The primary key value of one of this table's rows a strategy removes, under which a
    /// referencing table returns the entries its own strategies produced for that row.
    pub fn primary_key_value_of_a_row_to_delete() -> Ident {
        format_ident!("primary_key_value_of_a_row_to_delete")
    }

    /// The entries of the tables this table's removed rows cascaded into, by row.
    pub fn child_entries_by_primary_key_value_of_row_to_delete() -> Ident {
        format_ident!("child_entries_by_primary_key_value_of_row_to_delete")
    }
}
