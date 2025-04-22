use crate::{
    input::{ColumnSchema, TableSchema},
    output::{into_option, is_option},
};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::output::{get_column_type, get_column_value};

pub fn build(table: &TableSchema) -> TokenStream {
    let mut traits: Vec<TokenStream> = vec![];

    table.columns.iter().for_each(|column| {
        traits.push(get_one_row_option_by(table, column));
    });

    quote! {
        #(#traits)*
    }
}

fn get_one_row_option_by(table: &TableSchema, column: &ColumnSchema) -> TokenStream {
    if !column.is_primary_key && !column.has_unique_constraint {
        return TokenStream::default();
    }

    let trait_name = format_ident!(
        "Get{}RowOptionBy{}",
        &table.struct_name,
        RenameRule::PascalCase.apply_to_field(column.column_name.to_string())
    );
    let comment = format!(
        "Get a Option<{}> by it's {}.",
        &table.struct_name, &column.column_name
    );
    let method_name = format_ident!(
        "get_{}_by_{}",
        RenameRule::SnakeCase.apply_to_variant(table.struct_name.to_string()),
        &column.column_name
    );
    let column_name = &column.column_name;
    let column_type = get_column_type(column);
    let struct_name = &table.struct_name;
    let table_name = &table.singular_table_name;
    let column_value = get_column_value(column);

    let option_wrapper;

    if column.column_type_wrapper.is_some() && is_option(column) {
        option_wrapper = into_option(column);
    } else {
        option_wrapper = TokenStream::default();
    }

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name(
                &self,
                #column_name: #column_type,
            ) -> Option<#struct_name> {
                #option_wrapper
                return self
                        .ctx()
                        .db()
                        .#table_name()
                        .#column_name()
                        .find(#column_value);
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
