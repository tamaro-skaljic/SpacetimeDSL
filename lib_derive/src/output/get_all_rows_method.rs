use crate::input::TableSchema;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build(table: &TableSchema) -> TokenStream {
    let trait_name = format_ident!("GetAll{}Rows", &table.struct_name,);
    let comment = format!("Get all {}.", &table.struct_name,);
    let method_name = format_ident!("get_all_{}", &table.plural_table_name,);
    let struct_name = &table.struct_name;
    let table_name = &table.singular_table_name;

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name<'a>(
                &'a self,
            ) -> impl Iterator<Item = #struct_name> {
                return self
                    .ctx()
                    .db()
                    .#table_name()
                    .iter();
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
