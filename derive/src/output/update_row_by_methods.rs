use crate::input::{ColumnSchema, TableSchema};
use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Visibility;

pub fn build(table: &TableSchema) -> TokenStream {
    let mut is_mutable: bool = false;
    let mut has_modified_at: bool = false;

    table.columns.iter().for_each(|c| {
        if c.column_name.to_string().eq("modified_at") {
            has_modified_at = true;
        }

        match &c.visibility {
            Visibility::Inherited => {}
            _ => {
                is_mutable = true;
            }
        }
    });

    if !is_mutable {
        return TokenStream::default();
    }

    let mut traits: Vec<TokenStream> = vec![];

    table.columns.iter().for_each(|column| {
        traits.push(update_one_row_by(table, column, has_modified_at));
    });

    quote! {
        #(#traits)*
    }
}

fn update_one_row_by(
    table: &TableSchema,
    column: &ColumnSchema,
    has_modified_at: bool,
) -> TokenStream {
    if !column.is_primary_key && !column.has_unique_constraint {
        return TokenStream::default();
    }

    let trait_name = format_ident!(
        "Update{}RowBy{}",
        &table.struct_name,
        RenameRule::PascalCase.apply_to_field(column.column_name.to_string())
    );
    let comment = format!(
        "Update a {} row by it's {}.",
        &table.struct_name, &column.column_name
    );
    let method_name = format_ident!(
        "update_{}_by_{}",
        RenameRule::SnakeCase.apply_to_variant(table.struct_name.to_string()),
        &column.column_name
    );
    let column_name = &column.column_name;
    let struct_name = &table.struct_name;
    let table_name = &table.singular_table_name;

    let modified_at;

    if has_modified_at {
        modified_at = quote! {
            #table_name.modified_at = self.ctx().timestamp;
        }
    } else {
        modified_at = TokenStream::default();
    }

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name(
                &self,
                mut #table_name: #struct_name,
            ) -> #struct_name {
                #modified_at
                return self
                        .ctx()
                        .db()
                        .#table_name()
                        .#column_name()
                        .update(#table_name);
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
