use ident_case::RenameRule;
use proc_macro2::Ident;
use quote::format_ident;
use syn::{
    Data, DeriveInput, Fields, Path, Type, TypePath
};

use super::ColumnSchema;

pub fn get(
    syntax_tree: &DeriveInput,
    table_name: &Ident,
) -> Vec<ColumnSchema> {
    let mut columns: Vec<ColumnSchema> = Vec::new();

    match &syntax_tree.data {
        Data::Struct(st) => match &st.fields {
            Fields::Named(fields) => {
                fields.named.iter().for_each(|field| {
                let column_name = 
                    field.ident.clone().expect(&format!("The column must have a name. (Table: {})", table_name))
                .clone();
                let column_type = field.ty.clone();
                let visibility = field.vis.clone();
                let mut column_type_wrapper: Option<Type> = None;
                let mut is_primary_key = false;
                let mut has_unique_constraint = false;
                let mut has_single_column_index = false;
                let mut is_auto_inc = false;

                for attr in field.attrs.iter() {
                    let ident = &attr
                        .meta
                        .path()
                        .segments
                        .get(0)
                        .expect(&format!("The column must have at least one path segment. (Table: {}, Column: {})", table_name, column_name.to_string()))
                        .ident;
                    if ident.eq(&Ident::new("primary_key", ident.span())){
                        is_primary_key = true;
                    } else if ident.eq(&Ident::new("unique", ident.span())) {
                        has_unique_constraint = true;
                    } else if ident.eq(&Ident::new("index", ident.span())) {
                        has_single_column_index = true;
                    } else if ident.eq(&Ident::new("wrap", ident.span())) {
                        match attr.parse_args::<Type>() {
                            Ok(type_wrapper) => {
                                column_type_wrapper = Some(type_wrapper.clone());
                            },
                            Err(_) => {
                                column_type_wrapper = Some(Type::Path(TypePath {qself: None, path: Path::from(
                                    format_ident!(
                                        "{}{}",
                                        RenameRule::PascalCase.apply_to_field(table_name.to_string()),
                                        RenameRule::PascalCase.apply_to_field(column_name.to_string()),
                                    )
                                )}));
                            },
                        }
                    } else if ident.eq(&Ident::new("auto_inc", ident.span())) {
                        is_auto_inc = true;
                    }
                }

                if is_primary_key.eq(&true) && (has_unique_constraint.eq(&true) || has_single_column_index.eq(&true)) {
                    panic!("It makes no sense to annotate a column with #[index] which is already annotated with #[primary_key] or #[unique]. (Table: {}, Column: {})", table_name, column_name.to_string())
                }

                columns.push(ColumnSchema {
                    column_name,
                    column_type,
                    visibility,
                    column_type_wrapper,
                    is_primary_key,
                    has_unique_constraint,
                    has_single_column_index,
                    is_auto_inc,
                });
            });
            }
            Fields::Unnamed(_) => unimplemented!(),
            Fields::Unit => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    columns
}
