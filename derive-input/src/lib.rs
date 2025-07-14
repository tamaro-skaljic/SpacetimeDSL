pub mod api {
    pub mod rust;

    pub mod db;

    pub mod dsl;

    /**
     * The representation of a Rust struct with `#[table]` and `#[dsl]` attribute macros and its columns.
     */

    pub struct Table {
        pub rust_struct: rust::table::RustStruct,
        pub spacetimedb_table: db::table::SpacetimeDBTable,
        pub spacetimedsl_table: dsl::table::SpacetimeDSLTable,
        pub columns: Vec<Column>,
        pub spacetimedsl_methods: dsl::table::SpacetimeDSLTableMethods,
    }

    impl Table {
        /**
         * Supply the &DeriveInput which you've got from your own [derive macro](https://doc.rust-lang.org/reference/procedural-macros.html#derive-macros)
         * to this function to build upon your SpacetimeDB rust server module with SpacetimeDSL.
         */
        pub fn try_parse(
            args: proc_macro2::TokenStream,
            input: &syn::DeriveInput,
        ) -> syn::Result<Table> {
            crate::internal::try_parse(args, input)
        }
    }

    /**
     * The representation of a field of a Rust struct with `#[table]` and `#[dsl]` attribute macros.
     */

    pub struct Column {
        pub rust_field: rust::column::RustField,
        pub spacetimedb_column: db::column::SpacetimeDBColumn,
        pub spacetimedsl_column: dsl::column::SpacetimeDSLColumn,
        pub spacetimedsl_methods: Option<dsl::column::SpacetimeDSLColumnMethods>,
    }
}

#[doc(hidden)]
mod internal;

#[cfg(test)]
mod tests {
    use super::api::Table;
    use quote::quote;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_multiple_table_attributes_with_name_matching() {
        // Test the original problem case mentioned in issue #10
        let input: DeriveInput = parse_quote! {
            #[spacetimedsl::dsl(plural_name = test_tables1)]
            #[spacetimedb::table(name = test_table1, public)]
            #[spacetimedsl::dsl(plural_name = test_tables2)]
            #[spacetimedb::table(name = test_table2, public)]
            pub struct TestStruct {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u128,
            }
        };

        // Test first DSL macro should match test_table1
        let dsl_args1 = quote! { plural_name = test_tables1 };
        let result1 = Table::try_parse(dsl_args1, &input);
        assert!(
            result1.is_ok(),
            "First DSL macro should process successfully"
        );

        // Test second DSL macro should match test_table2
        let dsl_args2 = quote! { plural_name = test_tables2 };
        let result2 = Table::try_parse(dsl_args2, &input);
        assert!(
            result2.is_ok(),
            "Second DSL macro should process successfully"
        );

        // The results should be for different tables
        let table1 = result1.unwrap();
        let table2 = result2.unwrap();

        // Extract table names to verify they're different
        let table1_name = table1.spacetimedb_table.singular_name.to_string();
        let table2_name = table2.spacetimedb_table.singular_name.to_string();

        assert!(
            table1_name.contains("test_table1"),
            "First result should be for test_table1"
        );
        assert!(
            table2_name.contains("test_table2"),
            "Second result should be for test_table2"
        );
        assert_ne!(
            table1_name, table2_name,
            "Results should be for different tables"
        );
    }

    #[test]
    fn test_single_table_attribute_still_works() {
        // Ensure we didn't break the single table case
        let input: DeriveInput = parse_quote! {
            #[spacetimedsl::dsl(plural_name = entities)]
            #[spacetimedb::table(name = entity, public)]
            pub struct Entity {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u128,
            }
        };

        let dsl_args = quote! { plural_name = entities };
        let result = Table::try_parse(dsl_args, &input);
        assert!(result.is_ok(), "Single table case should still work");
    }

    #[test]
    fn test_fallback_deterministic_selection() {
        // Test the hash-based fallback when name matching doesn't work
        let input: DeriveInput = parse_quote! {
            #[spacetimedsl::dsl(plural_name = things)]
            #[spacetimedb::table(name = table_a, public)]
            #[spacetimedsl::dsl(plural_name = objects)]
            #[spacetimedb::table(name = table_b, public)]
            pub struct TestStruct {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u128,
            }
        };

        // Both DSL calls should succeed
        let dsl_args1 = quote! { plural_name = things };
        let result1 = Table::try_parse(dsl_args1.clone(), &input);
        assert!(
            result1.is_ok(),
            "First DSL macro should process successfully"
        );

        let dsl_args2 = quote! { plural_name = objects };
        let result2 = Table::try_parse(dsl_args2, &input);
        assert!(
            result2.is_ok(),
            "Second DSL macro should process successfully"
        );

        // Same DSL args should consistently select the same table (deterministic)
        let result1_repeat = Table::try_parse(dsl_args1, &input);
        assert!(result1_repeat.is_ok());

        let table1 = result1.unwrap();
        let table1_repeat = result1_repeat.unwrap();

        let table1_name = table1.spacetimedb_table.singular_name.to_string();
        let table1_repeat_name = table1_repeat.spacetimedb_table.singular_name.to_string();

        assert_eq!(
            table1_name, table1_repeat_name,
            "Same DSL args should select same table consistently"
        );
    }

    #[test]
    fn test_last_dsl_attribute_detection() {
        // Test the logic for detecting if a DSL attribute is the last one
        // This simulates what the actual macro would do
        
        let input: DeriveInput = parse_quote! {
            #[spacetimedsl::dsl(plural_name = test_tables1)]
            #[spacetimedb::table(name = test_table1, public)]
            #[spacetimedsl::dsl(plural_name = test_tables2)]
            #[spacetimedb::table(name = test_table2, public)]
            pub struct TestStruct {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u128,
            }
        };

        // Simulate the detection logic
        let mut dsl_attributes = Vec::new();
        
        // Find all dsl attributes on the struct
        for attr in &input.attrs {
            if let syn::Meta::List(meta_list) = &attr.meta {
                if meta_list.path.segments.len() == 2 
                    && meta_list.path.segments[0].ident == "spacetimedsl"
                    && meta_list.path.segments[1].ident == "dsl" {
                    dsl_attributes.push(&meta_list.tokens);
                }
            }
        }

        // We should find exactly 2 DSL attributes
        assert_eq!(dsl_attributes.len(), 2, "Should find exactly 2 DSL attributes");

        // Test first DSL attribute (should not be last)
        let first_args = quote! { plural_name = test_tables1 };
        let first_args_str = first_args.to_string();
        let mut first_found_index = None;
        
        for (index, attr_tokens) in dsl_attributes.iter().enumerate() {
            if attr_tokens.to_string() == first_args_str {
                first_found_index = Some(index);
                break;
            }
        }
        
        let is_first_last = match first_found_index {
            Some(index) => index == dsl_attributes.len() - 1,
            None => true,
        };
        
        assert!(!is_first_last, "First DSL attribute should not be detected as last");

        // Test second DSL attribute (should be last)
        let second_args = quote! { plural_name = test_tables2 };
        let second_args_str = second_args.to_string();
        let mut second_found_index = None;
        
        for (index, attr_tokens) in dsl_attributes.iter().enumerate() {
            if attr_tokens.to_string() == second_args_str {
                second_found_index = Some(index);
                break;
            }
        }
        
        let is_second_last = match second_found_index {
            Some(index) => index == dsl_attributes.len() - 1,
            None => true,
        };
        
        assert!(is_second_last, "Second DSL attribute should be detected as last");
    }

    #[test]
    fn test_single_dsl_attribute_is_last() {
        // Test single DSL attribute case (should always be considered last)
        let input: DeriveInput = parse_quote! {
            #[spacetimedsl::dsl(plural_name = entities)]
            #[spacetimedb::table(name = entity, public)]
            pub struct Entity {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u128,
            }
        };

        let mut dsl_attributes = Vec::new();
        
        // Find all dsl attributes on the struct
        for attr in &input.attrs {
            if let syn::Meta::List(meta_list) = &attr.meta {
                if meta_list.path.segments.len() == 2 
                    && meta_list.path.segments[0].ident == "spacetimedsl"
                    && meta_list.path.segments[1].ident == "dsl" {
                    dsl_attributes.push(&meta_list.tokens);
                }
            }
        }

        // Should find exactly 1 DSL attribute, and it should be considered last
        assert_eq!(dsl_attributes.len(), 1, "Should find exactly 1 DSL attribute");
        
        let is_last = dsl_attributes.len() <= 1;
        assert!(is_last, "Single DSL attribute should be detected as last");
    }
}
