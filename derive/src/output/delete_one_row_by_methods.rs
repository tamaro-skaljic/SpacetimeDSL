use crate::input::{Column, Table};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::output::{get_column_type, get_column_value};

pub fn build(table: &Table) -> TokenStream {
    let mut traits: Vec<TokenStream> = vec![];

    table.columns.iter().for_each(|column| {
        traits.push(delete_one_row_by(table, column));
    });

    quote! {
        #(#traits)*
    }
}

fn delete_one_row_by(table: &Table, column: &Column) -> TokenStream {
    if !column.is_primary_key && !column.has_unique_constraint {
        return TokenStream::default();
    }

    let trait_name = format_ident!(
        "Delete{}RowBy{}",
        &table.struct_name,
        RenameRule::PascalCase.apply_to_field(column.column_name.to_string())
    );
    let comment = format!(
        "Delete a {} row by it's {} (or None).",
        &table.struct_name, &column.column_name
    );
    let method_name = format_ident!(
        "delete_{}_by_{}",
        RenameRule::SnakeCase.apply_to_variant(table.struct_name.to_string()),
        &column.column_name
    );
    let column_name = &column.column_name;
    let column_type = get_column_type(column);
    let table_name = &table.singular_table_name;
    let column_value = get_column_value(column);

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name(
                &self,
                #column_name: #column_type,
            ) -> bool {
                return self
                        .ctx()
                        .db()
                        .#table_name()
                        .#column_name()
                        .delete(#column_value);
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
