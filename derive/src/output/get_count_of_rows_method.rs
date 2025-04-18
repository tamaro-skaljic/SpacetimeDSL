use crate::input::TableSchema;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build(table: &TableSchema) -> TokenStream {
    let trait_name = format_ident!("GetCountOf{}Rows", &table.struct_name,);
    let comment = format!("Get the count of all {}.", &table.struct_name,);
    let method_name = format_ident!("get_count_of_{}", &table.plural_table_name,);
    let table_name = &table.singular_table_name;

    quote! {
        pub trait #trait_name: spacetimedsl::DSLContext {
            #[doc=#comment]
            fn #method_name<'a>(
                &'a self,
            ) -> u64 {
                return self
                    .ctx()
                    .db()
                    .#table_name()
                    .count();
            }
        }

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
