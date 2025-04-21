use proc_macro2::Ident;
use syn::{DeriveInput, Type, Visibility};

mod columns;
mod table_name;

pub struct TableSchema {
    pub singular_table_name: Ident,
    pub plural_table_name: Ident,
    pub struct_name: Ident,
    pub columns: Vec<ColumnSchema>,
}

pub struct ColumnSchema {
    pub column_name: Ident,
    pub column_type: Type,
    pub visibility: Visibility,
    pub column_type_wrapper: Option<Type>,
    pub is_primary_key: bool,
    pub has_unique_constraint: bool,
    pub has_single_column_index: bool,
    pub is_auto_inc: bool,
}

pub fn parse(syntax_tree: DeriveInput) -> Option<TableSchema> {
    let (singular_table_name, plural_table_name) = table_name::get(&syntax_tree)?;
    let struct_name = syntax_tree.ident.clone();
    let columns = columns::get(&syntax_tree, &singular_table_name);

    Some(TableSchema {
        singular_table_name,
        plural_table_name,
        struct_name,
        columns,
    })
}
