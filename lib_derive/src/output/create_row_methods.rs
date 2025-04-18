use crate::input::TableSchema;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn build(table: &TableSchema) -> TokenStream {
    let table_name = &table.singular_table_name;
    let struct_name = &table.struct_name;
    let trait_name = format_ident!("Create{struct_name}");
    let comment = format!("Create a {struct_name}.");
    let method_name = format_ident!("create_{table_name}");
    let try_insert_error_generic_type = format_ident!("{table_name}__TableHandle");

    let trait_definition;

    if table.no_args_constructor.eq(&true) {
        trait_definition = quote! {
            pub trait #trait_name: spacetimedsl::DSLContext {
                #[doc=#comment]
                fn #method_name(
                    &self,
                ) -> Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>> {
                    return self
                            .ctx()
                            .db()
                            .#table_name()
                            .try_insert(#struct_name::new());
                }
            }
        }
    } else {
        trait_definition = quote! {
            pub trait #trait_name: spacetimedsl::DSLContext {
                #[doc=#comment]
                fn #method_name(
                    &self,
                    #table_name: #struct_name,
                ) -> Result<#struct_name, spacetimedb::TryInsertError<#try_insert_error_generic_type>> {
                    return self
                            .ctx()
                            .db()
                            .#table_name()
                            .try_insert(#table_name);
                }
            }
        }
    }
    quote! {
        #trait_definition

        impl #trait_name for spacetimedsl::DSL<'_> {}
    }
}
