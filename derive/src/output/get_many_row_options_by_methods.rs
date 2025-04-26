use crate::input::{Column, Table};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::output::{get_column_type, get_column_value};

pub fn build(table: &Table) -> TokenStream {
    let mut traits: Vec<TokenStream> = vec![];

    table.columns.iter().for_each(|column| {
        traits.push(get_many_row_options_by(table, column));
    });

    quote! {
        #(#traits)*
    }
}

fn get_many_row_options_by(table: &Table, column: &Column) -> TokenStream {
    if !column.is_primary_key && !column.has_unique_constraint {
        return TokenStream::default();
    }

    let trait_name = format_ident!(
        "Get{}RowOptionsBy{}",
        &table.struct_name,
        RenameRule::PascalCase.apply_to_field(column.column_name.to_string())
    );
    let comment = format!(
        "Get all Option<{}> by it's {}'s.",
        &table.struct_name, &column.column_name
    );
    let method_name = format_ident!(
        "get_{}_by_{}_in",
        &table.plural_table_name,
        &column.column_name
    );
    let parameter_name = format_ident!("{}s", &column.column_name);
    let column_name = &column.column_name;
    let column_type = get_column_type(column);
    let struct_name = &table.struct_name;
    let table_name = &table.singular_table_name;
    let column_value = get_column_value(column);

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name<'a>(
                &'a self,
                #parameter_name: Vec<#column_type>,
            ) -> impl Iterator<Item = Option<#struct_name>> {
                let mut result: Vec<Option<#struct_name>> = vec![];

                for #column_name in #parameter_name {
                    result.push(self.ctx().db().#table_name().#column_name().find(#column_value));
                }

                result.into_iter()
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
