use crate::api::db::{index::IndexType, table::SpacetimeDBTable};
use crate::api::dsl::reference::ReferencingTable;
use crate::api::dsl::table::SpacetimeDSLTable;
use proc_macro2::Span;
use quote::ToTokens;
use spacetime_bindings_macro_input::table::ColumnArgs;
use syn::Ident;

impl SpacetimeDSLTable {
    pub(in crate::internal) fn try_parse(
        column_args: &ColumnArgs<'_>,
        mut spacetimedb_table: SpacetimeDBTable,
        name_plural: Ident,
        unique_indices: Vec<syn::Ident>,
    ) -> syn::Result<(SpacetimeDBTable, SpacetimeDSLTable)> {
        for unique_index_name in unique_indices {
            for multi_column_index in &mut spacetimedb_table.multi_column_indices {
                match &multi_column_index.index_type {
                    IndexType::BTreeMultiColumn { columns: _ } => {
                        if multi_column_index.name.eq(&unique_index_name) {
                            multi_column_index.is_unique = true;
                        }
                    }
                    _ => {}
                }
            }
        }

        match column_args
            .primary_key_column
            .expect("The table should have a `#[primary_key]` column!")
            .vis
        {
            syn::Visibility::Public(_) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "A `#[primary_key]` column should have `Visibility::Inherited`! Found: Visibility::Public",
                ));
            }
            syn::Visibility::Restricted(_) => {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "A `#[primary_key]` column should have `Visibility::Inherited`! Found: Visibility::Restricted",
                ));
            }
            syn::Visibility::Inherited => {}
        }

        let mut is_mutable = false;
        let mut has_created_at_column = false;
        let mut has_modified_at_column = false;
        let mut referencing_tables = vec![];
        for field in &column_args.fields {
            let refs = ReferencingTable::try_parse(field)?;
            if referencing_tables.is_empty() {
                referencing_tables = refs;
            }

            if !is_mutable {
                is_mutable = match field.vis {
                    syn::Visibility::Public(_) => true,
                    syn::Visibility::Restricted(_) => true,
                    _ => false,
                };
            }
            if !has_created_at_column {
                if field
                    .name
                    .as_ref()
                    .expect("should have a name")
                    .eq("created_at")
                {
                    let field_type = field.ty.to_token_stream().to_string();
                    if !field_type.eq("Timestamp") && !field_type.eq("spacetimedb :: Timestamp") {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!(
                                "A column with name `created_at` should have the type `spacetimedb::Timestamp`! Found: {field_type}"
                            ),
                        ));
                    }

                    match field.vis {
                        syn::Visibility::Public(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `created_at` should have `Visibility::Inherited`! Found: Visibility::Public",
                            ));
                        }
                        syn::Visibility::Restricted(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `created_at` should have `Visibility::Inherited`! Found: Visibility::Restricted",
                            ));
                        }
                        syn::Visibility::Inherited => {
                            has_created_at_column = true;
                        }
                    }
                }
            }
            if !has_modified_at_column {
                if field
                    .name
                    .as_ref()
                    .expect("should have a name")
                    .eq("modified_at")
                {
                    let field_type = field.ty.to_token_stream().to_string();
                    // TODO: Allow Option<Timestamp> as modified_at column type
                    if !field_type.eq("Timestamp") && !field_type.eq("spacetimedb :: Timestamp") {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!(
                                "A column with name `modified_at` should have the type `spacetimedb::Timestamp`! Found: {field_type}"
                            ),
                        ));
                    }

                    match field.vis {
                        syn::Visibility::Public(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `modified_at` should have `Visibility::Inherited`! Found: Visibility::Public",
                            ));
                        }
                        syn::Visibility::Restricted(_) => {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "A column with name `modified_at` should have `Visibility::Inherited`! Found: Visibility::Restricted",
                            ));
                        }
                        syn::Visibility::Inherited => {
                            has_modified_at_column = true;
                        }
                    }
                }
            }
        }

        Ok((
            spacetimedb_table,
            SpacetimeDSLTable {
                plural_name: name_plural,
                is_mutable,
                has_created_at_column,
                has_modified_at_column,
                referencing_tables,
            },
        ))
    }
}
