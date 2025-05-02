use crate::input::Table;
use proc_macro2::TokenStream;
use quote::quote;
use spacetimedsl_derive_output::api::dsl::column::WrapperType;

pub fn build(table: &Table) -> TokenStream {
    let mut wrapper_types = vec![];

    table.columns.iter().for_each(|column| {
        match &column.spacetimedsl.wrapper_type {
            Some(wrapper_type) => match wrapper_type {
                WrapperType::Wrap(wrap) => {
                    wrapper_types.push(&wrap.wrapper_impl);
                }
                _ => {}
            },
            None => {}
        };
    });

    quote! {
        #(#wrapper_types)*
    }
}
