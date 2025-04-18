use crate::input::{ColumnSchema, TableSchema};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::output::{get_column_type, get_column_value};

pub fn build(table: &TableSchema) -> TokenStream {
    let mut traits: Vec<TokenStream> = vec![];

    table.columns.iter().for_each(|column| {
        traits.push(delete_many_rows_by(table, column));
    });

    quote! {
        #(#traits)*
    }
}

fn delete_many_rows_by(table: &TableSchema, column: &ColumnSchema) -> TokenStream {
    if !column.has_single_column_index {
        return TokenStream::default();
    }

    let trait_name = format_ident!(
        "Delete{}RowsBy{}",
        &table.struct_name,
        RenameRule::PascalCase.apply_to_field(column.column_name.to_string())
    );
    let comment = format!(
        "Delete all {} rows by it's {}.",
        &table.struct_name, &column.column_name
    );
    let method_name = format_ident!(
        "delete_{}_by_{}",
        &table.plural_table_name,
        &column.column_name
    );
    let column_name = &column.column_name;
    let column_type = get_column_type(column);
    let table_name = &table.singular_table_name;
    let column_value = get_column_value(column);

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name<'a>(
                &'a self,
                #column_name: &'a #column_type,
            ) -> u64 {
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
