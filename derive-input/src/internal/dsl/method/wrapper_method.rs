//! The methods a table adds to the wrapper types of its foreign key columns.
//!
//! Each one looks rows up through the DSL method of the column's index, so the lookup, its
//! name and its return type stay defined once, in `get.rs`.
//!
//! It also adds the methods in the other direction to the wrapper type of the table's primary
//! key, which look up the row a foreign key column references.

use {
    super::{context::MethodGenerationContext, foreign_key::foreign_key_of, naming},
    crate::{
        api::{
            Column,
            dsl::{
                column::SpacetimeDSLColumnMethods,
                wrapper::{WrapperMethod, WrapperType},
            },
            runtime,
        },
        internal::spacetimedb,
    },
    ident_case::RenameRule,
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    std::collections::BTreeMap,
    syn::{Ident, Type},
};

/// One method per column of `columns_with_foreign_key`, which all reference
/// `referenced_table_name`.
pub fn for_wrapper_methods(
    referenced_table_name: &syn::Ident,
    columns_with_foreign_key: &[&Column],
    context: &MethodGenerationContext,
) -> Vec<WrapperMethod> {
    let MethodGenerationContext {
        struct_name,
        singular_table_name,
        plural_table_name,
        ..
    } = context;

    let takes_name_of_dsl_method =
        referenced_table_name == singular_table_name || columns_with_foreign_key.len() > 1;

    columns_with_foreign_key
        .iter()
        .filter_map(|column| {
            let column_methods = column.spacetimedsl_methods.as_ref()?;

            let wrapper_type = column.spacetimedsl_column.wrapper_type.as_ref().expect(
                "`internal/dsl/column.rs` rejects a `#[foreign_key]` column without `#[use_wrapper]`",
            );

            let (dsl_method, short_method_name, rows_found, return_type, collect_rows) =
                match column_methods {
                SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => (
                    &methods.get_one_option,
                    format_ident!("get_{singular_table_name}"),
                    format!("the `{struct_name}`"),
                    methods.get_one_option.return_type.clone(),
                    TokenStream::default(),
                ),
                SpacetimeDSLColumnMethods::ForIndex(methods) => (
                    &methods.get_many,
                    format_ident!("get_{plural_table_name}"),
                    format!("all `{struct_name}` rows"),
                    quote! { Vec<#struct_name> },
                    quote! { .collect() },
                ),
            };

            let method_name = match takes_name_of_dsl_method {
                true => dsl_method.method_name.clone(),
                false => short_method_name,
            };

            let dsl_method_name = &dsl_method.method_name;
            let column_name = &column.rust_field.name;
            let wrapper_struct_name = wrapper_type.struct_name();
            let wrapper_variable_name =
                RenameRule::SnakeCase.apply_to_variant(wrapper_struct_name.to_string());

            Some(WrapperMethod {
                wrapper_type: wrapper_type_of(wrapper_type),
                doc_comment: format!(
                    "Get {rows_found} whose `{column_name}` column references this `{wrapper_struct_name}`.\n\nUse it like `{wrapper_variable_name}.{method_name}(&dsl)`."
                ),
                method_name,
                return_type,
                method_impl: quote! {
                    dsl.into().#dsl_method_name(self)#collect_rows
                },
            })
        })
        .collect()
}

/// One method per foreign key column besides the primary key, added to the wrapper type of
/// the primary key, which looks up the row the column of the row with that key references.
///
/// It takes the name of the referenced table, `get_<table>`. Where the table references that
/// table through several columns, or references itself, the table's name would not say which
/// column the method follows, so it takes the column's stem instead: `parent_folder_id` adds
/// `get_parent_folder`.
pub fn for_referenced_row_methods(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> Vec<WrapperMethod> {
    let MethodGenerationContext {
        spacetimedsl_table,
        primary_key_column,
        struct_name,
        singular_table_name,
        primary_key_column_name,
        ..
    } = context;

    // A singleton's injected primary key has no wrapper type to add a method to.
    if spacetimedsl_table.is_singleton() {
        return vec![];
    }

    let primary_key_wrapper = primary_key_column
        .spacetimedsl_column_wrapper_type
        .as_ref()
        .expect(
            "`internal/dsl/column.rs` rejects a primary key without a wrapper outside singletons",
        );
    let primary_key_wrapper_struct_name = primary_key_wrapper.struct_name();
    let primary_key_wrapper_variable_name =
        RenameRule::SnakeCase.apply_to_variant(primary_key_wrapper_struct_name.to_string());
    let get_this_row =
        naming::get_by_index_method_name(singular_table_name, primary_key_column_name);

    let mut methods = vec![];

    for (referenced_table_name, columns_with_foreign_key) in foreign_key_columns_by_referenced_table
    {
        let takes_the_column_stem =
            *referenced_table_name == singular_table_name || columns_with_foreign_key.len() > 1;

        // A foreign key on the primary key references the row its own value names, and
        // `referenced_row_method = false` switches the method off.
        let columns = columns_with_foreign_key.iter().filter(|column| {
            !column.spacetimedb_column.is_primary_key
                && foreign_key_of(column).referenced_row_method
        });

        for column in columns {
            let foreign_key = foreign_key_of(column);
            let column_name = &column.rust_field.name;

            let method_name = match takes_the_column_stem {
                false => format_ident!("get_{referenced_table_name}"),
                true => format_ident!(
                    "get_{}",
                    column_stem(column_name, &foreign_key.primary_key_column_name)
                ),
            };

            let get_referenced_row = naming::get_by_index_method_name(
                &foreign_key.table_name,
                &foreign_key.primary_key_column_name,
            );
            let getter_name = naming::getter_name(column_name);
            let referenced_row_type =
                spacetimedb::table_row_type(&foreign_key.path, &foreign_key.table_name);

            methods.push(WrapperMethod {
                wrapper_type: wrapper_type_of(primary_key_wrapper),
                doc_comment: format!(
                    "Get the row of the `{referenced_table_name}` table which the `{column_name}` column of the `{struct_name}` row with this `{primary_key_wrapper_struct_name}` references.\n\nUse it like `{primary_key_wrapper_variable_name}.{method_name}(&dsl)`."
                ),
                method_name,
                return_type: runtime::error_result_type(&referenced_row_type),
                method_impl: quote! {
                    let dsl = dsl.into();
                    let #singular_table_name = dsl.#get_this_row(self)?;
                    dsl.#get_referenced_row(#singular_table_name.#getter_name())
                },
            });
        }
    }

    methods
}

/// The name of a foreign key column without the suffix which names the key it holds: without
/// `_<primary key of the referenced table>`, or else without `_id`, or else whole.
fn column_stem(column_name: &Ident, referenced_primary_key_column_name: &Ident) -> String {
    let column_name = column_name.to_string();
    let key_suffix = format!("_{referenced_primary_key_column_name}");

    column_name
        .strip_suffix(key_suffix.as_str())
        .or_else(|| column_name.strip_suffix("_id"))
        .unwrap_or(&column_name)
        .to_string()
}

/// The wrapper type as the `Type` `WrapperMethod::wrapper_type` holds.
fn wrapper_type_of(wrapper_type: &WrapperType) -> Type {
    let wrapper_path = wrapper_type.wrapper_path();

    syn::parse_quote!(#wrapper_path)
}
