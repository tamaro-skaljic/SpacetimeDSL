//! Every diagnostic SpacetimeDSL reports for a table it rejects.
//!
//! Each rejection is written here exactly once, so the wording users read is reviewed in
//! one place rather than scattered through the parsers that detect it. The caller decides
//! whether the input is rejected and which tokens the diagnostic underlines; these
//! functions only build the message around what they are given.
//!
//! The `compile-tests` pin every message together with the span it underlines.

use {
    crate::{
        api::dsl::{soft_delete::SoftDeleteMarkerKind, table::SingletonKind},
        internal::dsl::column_role,
    },
    proc_macro2::Span,
    quote::ToTokens,
    syn::{Error, Ident, Type, Visibility, meta::ParseNestedMeta},
};

/// Why a singleton is never soft-deletable, shared by the three shapes it is rejected in.
const SINGLETON_IS_NEVER_SOFT_DELETABLE: &str = "A singleton holds one row which the DSL looks up by its injected primary key, so retiring that row would leave the table with a row no method can reach.";

/// A visibility the way the user wrote it: `` `pub` ``, `` `pub(crate)` ``,
/// `` `pub(in path)` ``, or "no visibility modifier" for a private field.
fn written_visibility(visibility: &Visibility) -> String {
    match visibility {
        Visibility::Public(_) => "`pub`".to_string(),
        Visibility::Restricted(restricted) => {
            let path = restricted
                .path
                .to_token_stream()
                .to_string()
                .replace(' ', "");

            match restricted.in_token {
                Some(_) => format!("`pub(in {path})`"),
                None => format!("`pub({path})`"),
            }
        }
        Visibility::Inherited => "no visibility modifier".to_string(),
    }
}

// Column types

pub fn unsupported_column_type(column_type: &Type) -> Error {
    let kind = match column_type {
        Type::Array(_) => "an array",
        Type::Tuple(_) => "a tuple",
        Type::Reference(_) => "a reference",
        Type::Slice(_) => "a slice",
        Type::FnPtr(_) => "a function pointer",
        Type::TraitObject(_) => "a trait object",
        Type::ImplTrait(_) => "an `impl Trait`",
        Type::Never(_) => "the never type",
        Type::Path(_) => "a qualified path such as `<T as Trait>::Type`",
        Type::Ptr(_) => "a raw pointer",
        Type::Infer(_) => "the inferred type `_`",
        Type::Macro(_) => "a macro",
        _ => "not a path",
    };

    Error::new_spanned(
        column_type,
        format!(
            "SpacetimeDSL supports only path types as column types, such as `u64`, `String` or `spacetimedb::Timestamp`! This column's type is {kind}."
        ),
    )
}

// `#[table]`

pub fn missing_table_attribute(struct_name: &Ident) -> Error {
    Error::new_spanned(
        struct_name,
        "Haven't found a `#[table]`/`#[spacetimedb::table]` attribute on this struct! `#[dsl]`/`#[spacetimedsl::dsl]` builds on the table it declares.",
    )
}

pub fn table_selector_missing(struct_name: &Ident, accessors: &[&Ident]) -> Error {
    Error::new_spanned(
        struct_name,
        format!(
            "This struct has {} `#[table]` attributes ({}), so `#[dsl]` has to name the one it belongs to! Add `table = <accessor>`, such as `#[dsl(table = {}, ...)]`.",
            accessors.len(),
            quoted_list(accessors),
            accessors[0],
        ),
    )
}

pub fn table_selector_names_no_table(table_selector: &Ident, accessors: &[&Ident]) -> Error {
    Error::new_spanned(
        table_selector,
        format!(
            "No `#[table]` attribute of this struct has the accessor `{table_selector}`! Found: {}.",
            quoted_list(accessors),
        ),
    )
}

fn quoted_list(idents: &[&Ident]) -> String {
    idents
        .iter()
        .map(|ident| format!("`{ident}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn singleton_without_exactly_one_table_attribute(
    struct_name: &Ident,
    count_of_table_attributes: usize,
) -> Error {
    Error::new_spanned(
        struct_name,
        format!(
            "Singleton tables must have exactly one `#[table]` attribute, but found {count_of_table_attributes}!"
        ),
    )
}

pub fn multi_column_index_on_singleton(index_name: &Ident, column_names: &[Ident]) -> Error {
    Error::new_spanned(
        index_name,
        format!(
            "Multi-column indices are not allowed on singleton tables! Found index `{}` on columns `{}`.",
            index_name,
            column_names
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
    )
}

// `#[dsl(plural_name)]` and `#[dsl(unique_index)]`

pub fn missing_plural_name(dsl_arguments: &impl ToTokens) -> Error {
    Error::new_spanned(
        dsl_arguments,
        "PluralName must be set in `#[dsl(plural_name = PluralName)]`",
    )
}

pub fn plural_name_on_singleton(plural_name: &Ident) -> Error {
    Error::new_spanned(
        plural_name,
        "`plural_name` is not allowed on singleton tables! Use `#[dsl(singleton)]` without `plural_name`.",
    )
}

/// Built by [`ParseNestedMeta::error`], which underlines `unique_index` together with its
/// arguments.
pub fn missing_unique_index_name(meta: &ParseNestedMeta<'_>) -> Error {
    meta.error(
        "IndexName must be set in `#[dsl(unique_index(name = IndexName))]`, e.g. `name = my_index`.",
    )
}

pub fn unique_index_on_singleton(unique_index_name: &Ident, names_a_declared_index: bool) -> Error {
    let removal = match names_a_declared_index {
        true => format!(
            "Remove `unique_index(name = {unique_index_name})` and the `{unique_index_name}` index from `#[table]`."
        ),
        false => format!("Remove `unique_index(name = {unique_index_name})`."),
    };

    Error::new_spanned(
        unique_index_name,
        format!("`unique_index` is not allowed on singleton tables! {removal}"),
    )
}

pub fn unknown_unique_index(unique_index_name: &Ident, declared_indices: &[&Ident]) -> Error {
    let declared = match declared_indices.is_empty() {
        true => "none".to_string(),
        false => quoted_list(declared_indices),
    };

    Error::new_spanned(
        unique_index_name,
        format!(
            "No index of this table has the accessor `{unique_index_name}`! Its indices: {declared}."
        ),
    )
}

pub fn repeated_unique_index(unique_index_name: &Ident) -> Error {
    Error::new_spanned(
        unique_index_name,
        format!(
            "`unique_index(name = {unique_index_name})` is given twice! Remove this one, or name another index."
        ),
    )
}

// `#[dsl(method(..))]`

pub fn missing_update_method_with_non_private_column(struct_name: &Ident) -> Error {
    Error::new_spanned(
        struct_name,
        "HasUpdateMethod must be set in `#[dsl(method(update = HasUpdateMethod))]`\nBecause you have at least one column which is not private, you should set `#[dsl(method(update = true))]`.\nIf, instead, you want immutable rows in this table which don't have setters and can't be updated, all columns must be private and you must specify `#[dsl(method(update = false))]`.",
    )
}

pub fn missing_update_method_with_only_private_columns(struct_name: &Ident) -> Error {
    Error::new_spanned(
        struct_name,
        format!(
            "HasUpdateMethod must be set in `#[dsl(method(update = HasUpdateMethod))]`, e.g. `update = false`.\nBecause all your columns are private, you should set `#[dsl(method(update = false))]`.\nIf, instead, you want mutable rows in this table which have setters and can be updated, at least one column must be non-private, carry `#[set_on_update]` or be named {}, and you must specify `#[dsl(method(update = true))]`.",
            column_role::slash_separated(&column_role::SET_ON_UPDATE_COLUMN_NAMES),
        ),
    )
}

pub fn non_private_column_without_update_method(visibility: &Visibility) -> Error {
    Error::new_spanned(
        visibility,
        format!(
            "All columns in a table with disabled `update` DSL method should be private! Found: {}",
            written_visibility(visibility)
        ),
    )
}

pub fn update_method_disabled_with_set_on_update_column(struct_name: &Ident) -> Error {
    let set_on_update_column_names =
        column_role::slash_separated(&column_role::SET_ON_UPDATE_COLUMN_NAMES);

    Error::new_spanned(
        struct_name,
        format!(
            "Because you have a column named {set_on_update_column_names}, you must specify `#[dsl(method(update = true))]`\nIf, instead, you want immutable rows in this table which don't have setters and can't be updated, all columns must be private, you must remove the {set_on_update_column_names} column and you must specify `#[dsl(method(update = false))]`."
        ),
    )
}

pub fn update_method_disabled_on_singleton_with_default(with_default: Span) -> Error {
    Error::new(
        with_default,
        "Cannot disable the `update` method with `#[dsl(method(update = false))]` on a table with `#[dsl(singleton(with_default))]`!\n`upsert_<table>` is the only method which writes the row of such a table, so the table could never hold one.",
    )
}

pub fn soft_delete_method_without_delete_method(soft_delete_method_path: &impl ToTokens) -> Error {
    Error::new_spanned(
        soft_delete_method_path,
        "`#[dsl(method(soft_delete = ...))]` requires `#[dsl(method(delete = ...))]` to be set as well, e.g. `method(delete = true, soft_delete = true)`.\nSoft deletion retires a row instead of removing it, which only says something next to a decision about whether the table removes rows at all.",
    )
}

// `#[dsl(hook(..))]`

pub fn before_update_hook_without_update_method(hook: Span) -> Error {
    Error::new(
        hook,
        "Cannot have a `before_update` hook when the `update` method is disabled with `#[dsl(method(update = false))]`",
    )
}

pub fn after_update_hook_without_update_method(hook: Span) -> Error {
    Error::new(
        hook,
        "Cannot have an `after_update` hook when the `update` method is disabled with `#[dsl(method(update = false))]`",
    )
}

pub fn before_delete_hook_without_delete_method(hook: Span) -> Error {
    Error::new(
        hook,
        "Cannot have a `before_delete` hook when the `delete` method is disabled with `#[dsl(method(delete = false))]`",
    )
}

pub fn after_delete_hook_without_delete_method(hook: Span) -> Error {
    Error::new(
        hook,
        "Cannot have an `after_delete` hook when the `delete` method is disabled with `#[dsl(method(delete = false))]`",
    )
}

pub fn before_soft_delete_hook_on_table_not_soft_deletable(hook: Span) -> Error {
    Error::new(
        hook,
        "Cannot have a `before_soft_delete` hook when the table is not soft-deletable. Enable it with `#[dsl(method(soft_delete = true))]`",
    )
}

pub fn after_soft_delete_hook_on_table_not_soft_deletable(hook: Span) -> Error {
    Error::new(
        hook,
        "Cannot have an `after_soft_delete` hook when the table is not soft-deletable. Enable it with `#[dsl(method(soft_delete = true))]`",
    )
}

// `#[primary_key]`, `#[index]` and `#[unique]`

pub fn missing_primary_key(struct_name: &Ident) -> Error {
    Error::new_spanned(
        struct_name,
        "Your table should have a `#[primary_key]` column!",
    )
}

pub fn primary_key_prefixed_with_table_name(
    column_name: &Ident,
    singular_table_name: &Ident,
) -> Error {
    Error::new_spanned(
        column_name,
        format!(
            "A #[primary_key] column must not be prefixed with the table's name! Use `{}` instead of `{}`.",
            column_name
                .to_string()
                .strip_prefix(&format!("{singular_table_name}_"))
                .unwrap_or("id"),
            column_name,
        ),
    )
}

pub fn single_column_index_on_singleton(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        format!(
            "`#[index]` and `#[unique]` are not allowed on singleton tables! Found index on column `{column_name}`.",
        ),
    )
}

pub fn multiple_single_column_indices_on_column(
    column_name: &Ident,
    first_index_name: &Ident,
    second_index_name: &Ident,
) -> Error {
    Error::new_spanned(
        second_index_name,
        format!(
            "The column `{column_name}` has two single-column indices, `{first_index_name}` and `{second_index_name}`! Its lookup methods come from one of them, so the other would generate nothing. Remove one."
        ),
    )
}

// `#[create_wrapper]` and `#[use_wrapper]`

pub fn primary_key_without_wrapper(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "A #[primary_key] column must be accompanied by `#[create_wrapper]` or `#[use_wrapper]`!",
    )
}

pub fn multiple_wrappers(wrapper_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        wrapper_attribute,
        "Only one of `#[create_wrapper]` or `#[use_wrapper]` is allowed per column!",
    )
}

pub fn missing_use_wrapper_path(use_wrapper_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        use_wrapper_meta,
        "PathToWrapperType must be set in `#[use_wrapper(PathToWrapperType)]`, e.g. `EntityId` or `crate::entity::EntityId`.",
    )
}

pub fn invalid_create_wrapper_name(create_wrapper_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        create_wrapper_meta,
        "Failed to parse NameForWrapperType in `#[create_wrapper(NameForWrapperType)]`. Expected a valid Rust ident like `EntityId`.",
    )
}

pub fn invalid_use_wrapper_path(use_wrapper_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        use_wrapper_meta,
        "Failed to parse PathToWrapperType in `#[use_wrapper(PathToWrapperType)]`. Expected a valid Rust path like `EntityId` or `crate::entity::EntityId`.",
    )
}

// `#[auto_gen]`

pub fn multiple_auto_gen_attributes(auto_gen_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        auto_gen_attribute,
        "Only one `#[auto_gen]` is allowed per column!",
    )
}

pub fn invalid_uuid_version(auto_gen_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        auto_gen_attribute,
        "Expected `#[auto_gen(v4)]` or `#[auto_gen(v7)]`!",
    )
}

pub fn auto_gen_on_singleton_with_default(auto_gen_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        auto_gen_attribute,
        "`#[auto_gen]` is not allowed on a `singleton(with_default)` table, because views aren't able to access UUID generators!",
    )
}

pub fn auto_gen_column_type_mismatch(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "A column with `#[auto_gen]` should have the type `spacetimedb::Uuid`! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn auto_gen_column_not_private(visibility: &Visibility) -> Error {
    Error::new_spanned(
        visibility,
        format!(
            "A column with `#[auto_gen]` should be private, because its value is generated and should never change! Found: {}",
            written_visibility(visibility)
        ),
    )
}

pub fn auto_gen_with_used_wrapper(auto_gen_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        auto_gen_attribute,
        "A column with `#[auto_gen]` must be accompanied by `#[create_wrapper]`, not `#[use_wrapper(...)]`!",
    )
}

pub fn auto_gen_without_wrapper(auto_gen_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        auto_gen_attribute,
        "A column with `#[auto_gen]` must be accompanied by `#[create_wrapper]`!",
    )
}

// `set_on_create` and `set_on_update`

pub fn set_on_create_and_set_on_update(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "A column cannot be both `set_on_create` and `set_on_update`.",
    )
}

pub fn bare_timestamp_on_singleton_with_default(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "A column on a `singleton(with_default)` table should have the type `Option<spacetimedb::Timestamp>`! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn multiple_set_on_create_columns(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "Multiple columns claim the `set_on_create` role! Only one column is allowed.",
    )
}

pub fn set_on_create_column_type_mismatch(
    column_type: &Type,
    singleton: Option<SingletonKind>,
) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "A column with the `set_on_create` role should have the type `{}`! Found: {}",
            match singleton {
                Some(SingletonKind::WithDefault) => "Option<spacetimedb::Timestamp>",
                _ => "spacetimedb::Timestamp",
            },
            column_type.to_token_stream()
        ),
    )
}

pub fn set_on_create_column_not_private(visibility: &Visibility) -> Error {
    Error::new_spanned(
        visibility,
        format!(
            "A column with the `set_on_create` role should have no visibility modifier! Found: {}",
            written_visibility(visibility)
        ),
    )
}

pub fn multiple_set_on_update_columns(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "Multiple columns claim the `set_on_update` role! Only one column is allowed.",
    )
}

pub fn set_on_update_column_without_update_method(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "A column with the `set_on_update` role requires the `update` method to be enabled in `#[dsl(method(update = true))]`!",
    )
}

pub fn set_on_update_column_type_mismatch(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "A column with the `set_on_update` role should have the type `spacetimedb::Timestamp` or `Option<spacetimedb::Timestamp>`! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn set_on_update_column_not_private(visibility: &Visibility) -> Error {
    Error::new_spanned(
        visibility,
        format!(
            "A column with the `set_on_update` role should have no visibility modifier! Found: {}",
            written_visibility(visibility)
        ),
    )
}

// Soft deletion

pub fn soft_delete_method_without_marker_column(
    soft_delete_method_argument: &impl ToTokens,
) -> Error {
    Error::new_spanned(
        soft_delete_method_argument,
        format!(
            "`#[dsl(method(soft_delete = true))]` requires a column which the soft deletion writes!\nName a column {} and give it the type `bool`, name a column {} and give it the type `Option<spacetimedb::Timestamp>`, or put `#[set_on_soft_delete]` on a column of either type.",
            column_role::or_separated(&column_role::SOFT_DELETE_FLAG_COLUMN_NAMES),
            column_role::or_separated(&column_role::SOFT_DELETE_TIMESTAMP_COLUMN_NAMES),
        ),
    )
}

pub fn marker_column_on_table_not_soft_deletable(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "This column claims the soft-delete marker role, but the table is not soft-deletable!\nAdd `#[dsl(method(soft_delete = true))]` to the table, or rename the column and remove `#[set_on_soft_delete]` from it.",
    )
}

pub fn marker_column_type_mismatch(column_type: &Type, kind: SoftDeleteMarkerKind) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "A column with the soft-delete marker role should have the type `{}`! Found: {}",
            match kind {
                SoftDeleteMarkerKind::Flag => "bool",
                SoftDeleteMarkerKind::Timestamp => "Option<spacetimedb::Timestamp>",
            },
            column_type.to_token_stream()
        ),
    )
}

pub fn set_on_soft_delete_column_type_mismatch(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "A column with `#[set_on_soft_delete]` should have the type `bool` or `Option<spacetimedb::Timestamp>`! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn multiple_marker_columns(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "Multiple columns claim the soft-delete marker role! Only one column is allowed.",
    )
}

pub fn marker_column_not_private(visibility: &Visibility) -> Error {
    Error::new_spanned(
        visibility,
        format!(
            "A column with the soft-delete marker role should have no visibility modifier! Found: {}\nOnly DSL methods are allowed to set this column, and they do it internally, so it has a getter but no setter.",
            written_visibility(visibility)
        ),
    )
}

pub fn soft_delete_method_on_singleton(soft_delete_method_argument: &impl ToTokens) -> Error {
    Error::new_spanned(
        soft_delete_method_argument,
        format!(
            "`#[dsl(method(soft_delete = true))]` is not allowed on a singleton table!\n{SINGLETON_IS_NEVER_SOFT_DELETABLE} Remove `soft_delete = true`."
        ),
    )
}

pub fn soft_delete_method_with_marker_column_on_singleton(
    soft_delete_method_argument: &impl ToTokens,
    marker_column_name: &str,
) -> Error {
    Error::new_spanned(
        soft_delete_method_argument,
        format!(
            "`#[dsl(method(soft_delete = true))]` is not allowed on a singleton table!\n{SINGLETON_IS_NEVER_SOFT_DELETABLE} Remove `soft_delete = true` and the `{marker_column_name}` column."
        ),
    )
}

pub fn marker_column_on_singleton(
    marker_column_identifier: &Ident,
    marker_column_name: &str,
) -> Error {
    Error::new_spanned(
        marker_column_identifier,
        format!(
            "This column claims the soft-delete marker role, but a singleton table is never soft-deletable!\n{SINGLETON_IS_NEVER_SOFT_DELETABLE} Remove the `{marker_column_name}` column."
        ),
    )
}

// `#[foreign_key]`

pub fn foreign_key_without_wrapper(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "A #[foreign_key] column must be accompanied by `#[use_wrapper]`!",
    )
}

pub fn foreign_key_with_created_wrapper(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "A #[foreign_key] column must be accompanied by `#[use_wrapper]`, not `#[create_wrapper]`!",
    )
}

pub fn foreign_key_without_index(foreign_key_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_attribute,
        "`#[foreign_key]` is only allowed in combination with `#[primary_key]`, `#[unique]` or `#[index]`!",
    )
}

pub fn multiple_foreign_keys(foreign_key_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_attribute,
        "`#[foreign_key]` is only allowed once per column!",
    )
}

pub fn missing_foreign_key_path(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "PathToTable must be set in `#[foreign_key(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`. Supply the path to the referenced table.",
    )
}

pub fn missing_foreign_key_table(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "TableName must be set in `#[foreign_key(table = TableName)]`, e.g. `table = my_table`. Supply the name of the referenced table.",
    )
}

pub fn missing_foreign_key_column(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "PrimaryKeyColumnName must be set in `#[foreign_key(column = PrimaryKeyColumnName)]`, e.g. `column = id`. Supply the name of the primary key column in the referenced table.",
    )
}

pub fn foreign_key_without_on_delete_strategy(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "A `#[foreign_key]` must set `on_delete`, `on_soft_delete`, or both, e.g. `on_delete = Delete`.\nSet `on_delete` when the referenced table has a delete method, set `on_soft_delete` when it is soft-deletable, and set both when it is both. The referenced table's own `#[referenced_by]` decides which of them is required; leaving out a required one is an unresolved import naming the field to add.",
    )
}

pub fn set_zero_strategy_on_private_column(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::SetZero` is only allowed on non-private columns, because setters are only generated for non-private columns (column-level mutability constraints)!",
    )
}

pub fn set_zero_strategy_on_unsupported_type(column_type: &Type) -> Error {
    Error::new_spanned(
        column_type,
        format!(
            "`OnDeleteStrategy::SetZero` is only allowed on unsigned integer and `Uuid` columns, which it sets to `0` or `Uuid::NIL`, the values that reference no row! Found: {}",
            column_type.to_token_stream()
        ),
    )
}

pub fn delete_strategy_without_delete_method(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::Delete` is only allowed when the table has a delete method (`#[dsl(method(delete = true))]`)!",
    )
}

pub fn soft_delete_strategy_on_table_not_soft_deletable(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::SoftDelete` is only allowed when this table is soft-deletable (`#[dsl(method(soft_delete = true))]`)!\nThe strategy retires the rows of this table, which needs a marker column for the DSL to write.",
    )
}

pub fn set_none_strategy_not_implemented(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "Because Option is currently not allowed on primary_key and unique/btree indices, `OnDeleteStrategy::SetNone` isn't implemented yet. `OnDeleteStrategy` must be one of `Error`, `Delete`, `SoftDelete`, `SetZero` or `Ignore` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.",
    )
}

pub fn unknown_on_delete_strategy(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy` must be one of `Error`, `Delete`, `SoftDelete`, `SetNone`, `SetZero` or `Ignore` in `#[foreign_key(on_delete = OnDeleteStrategy)]`, e.g. `on_delete = Delete`.",
    )
}

pub fn delete_strategy_in_on_soft_delete(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::Delete` is not allowed in `on_soft_delete`! Soft-deleting a row must not physically remove the rows which reference it. `on_soft_delete` must be one of `Error`, `SoftDelete` or `Ignore`.",
    )
}

pub fn set_zero_strategy_in_on_soft_delete(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::SetZero` is not allowed in `on_soft_delete`! Soft deletion preserves the row, so clearing the foreign key column would destroy what it preserved. `on_soft_delete` must be one of `Error`, `SoftDelete` or `Ignore`.",
    )
}

pub fn unknown_on_soft_delete_strategy(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy` must be one of `Error`, `SoftDelete` or `Ignore` in `#[foreign_key(on_soft_delete = OnDeleteStrategy)]`, e.g. `on_soft_delete = SoftDelete`.",
    )
}

pub fn foreign_key_columns_type_mismatch(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "All foreign key columns which reference the same primary key of another table should have the same type",
    )
}

pub fn foreign_key_columns_path_mismatch(column_name: &Ident) -> Error {
    Error::new_spanned(
        column_name,
        "All foreign key columns which reference the same primary key of another table should have the same path! Spell both paths the same way; a leading `::` makes no difference.",
    )
}

// `#[referenced_by]`

pub fn referenced_by_without_primary_key(referenced_by_attribute: &impl ToTokens) -> Error {
    Error::new_spanned(
        referenced_by_attribute,
        "`#[referenced_by]` is only allowed in combination with `#[primary_key]`!",
    )
}

pub fn referenced_by_without_delete_or_soft_delete_method(
    referenced_by_attribute: &impl ToTokens,
) -> Error {
    Error::new_spanned(
        referenced_by_attribute,
        "`#[referenced_by]` is only allowed when the table has a delete method (`#[dsl(method(delete = true))]`) or is soft-deletable (`#[dsl(method(soft_delete = true))]`)!\nThe on-delete strategies it declares run when a row of this table is deleted or soft-deleted, neither of which the DSL can do while both are disabled.",
    )
}

pub fn missing_referenced_by_path(referenced_by_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        referenced_by_meta,
        "PathToTable must be set in `#[referenced_by(path = PathToTable)]`, e.g. `path = crate::path::to::my::table`.",
    )
}

pub fn missing_referenced_by_table(referenced_by_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        referenced_by_meta,
        "TableName must be set in `#[referenced_by(table = TableName)]`, e.g. `table = my_table`.",
    )
}
