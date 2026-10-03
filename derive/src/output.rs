use {
    proc_macro2::TokenStream,
    quote::{format_ident, quote},
    spacetimedsl_derive_input::api::{
        Table,
        dsl::{
            column::SpacetimeDSLColumnMethods,
            method::{SpacetimeDSLArg, SpacetimeDSLMethod},
            table::CascadeEntryPoints,
            wrapper::{WrapperMethod, WrapperType},
        },
    },
    syn::Ident,
};

mod accessor;
mod create_method_arg;
mod doc_comment;
mod function;
mod hook;
mod wrapper_method;

/// The generated output, split so callers can inspect every DSL method on its own
/// instead of walking the [`Table`] a second time themselves.
pub struct GeneratedOutput {
    /// Compile-error checks, wrapper types, the accessor `impl`, the create-argument struct and
    /// the hook traits - everything the macro emits that is neither a DSL method nor a wrapper method.
    pub items_outside_dsl_methods: TokenStream,
    pub dsl_methods: Vec<GeneratedDSLMethod>,
    /// The `impl` blocks which add lookup methods to the wrapper types of foreign key columns.
    pub wrapper_methods: TokenStream,
}

impl GeneratedOutput {
    /// Concatenates the halves in the order the macro emits them.
    pub fn into_token_stream(self) -> TokenStream {
        let items_outside_dsl_methods = self.items_outside_dsl_methods;
        let dsl_methods = self
            .dsl_methods
            .into_iter()
            .map(|dsl_method| dsl_method.tokens);
        let wrapper_methods = self.wrapper_methods;

        quote! {
            #items_outside_dsl_methods

            #(#dsl_methods)*

            #wrapper_methods
        }
    }
}

pub struct GeneratedDSLMethod {
    /// Only read by the characterization tests, which snapshot each method under its own name.
    #[cfg_attr(not(test), allow(dead_code))]
    pub method_name: Ident,
    /// Whether the macro generates this method for its own use instead of for the DSL user.
    /// Only read by the characterization tests, which snapshot the internal methods of a
    /// struct into one file because their generated names are too long to be file names.
    #[cfg_attr(not(test), allow(dead_code))]
    pub is_internal: bool,
    pub tokens: TokenStream,
}

/// Which of the struct's `#[dsl]` attributes an expansion is.
///
/// Every table of a struct shares the struct's wrapper types and accessors, so only its first
/// expansion emits them. A method on the primary key's wrapper type which reads a row of the
/// table would be ambiguous on a struct with several tables, because the key names a row in
/// each of them, so only the expansion of a struct's only `#[dsl]` emits those.
#[derive(Clone, Copy)]
pub enum DSLAttributePass {
    /// The struct's only `#[dsl]` attribute.
    Only,
    /// The first of several.
    First,
    /// A later one of several.
    Later,
}

impl DSLAttributePass {
    pub fn of(is_first: bool, is_last: bool) -> DSLAttributePass {
        match (is_first, is_last) {
            (true, true) => DSLAttributePass::Only,
            (true, false) => DSLAttributePass::First,
            (false, _) => DSLAttributePass::Later,
        }
    }

    fn emits_the_struct_items(self) -> bool {
        matches!(self, DSLAttributePass::Only | DSLAttributePass::First)
    }

    fn emits_the_referenced_row_methods(self) -> bool {
        matches!(self, DSLAttributePass::Only)
    }
}

pub fn build(input: &Table, pass: DSLAttributePass) -> syn::Result<GeneratedOutput> {
    let struct_name = format_ident!("{}", &input.rust_struct.name.to_string());
    let mut wrapper_types = vec![];

    // Every table of the struct shares its wrapper types, so the first expansion emits them.
    if pass.emits_the_struct_items() {
        for column in &input.columns {
            if let Some(WrapperType::Created(wrapper_type)) =
                &column.spacetimedsl_column.wrapper_type
            {
                wrapper_types.push(&wrapper_type.wrapper_impl);
            }
        }
    }

    let mut table_methods = vec![];
    let mut dsl_methods = vec![];

    if let Some(method) = &input.spacetimedsl_methods.create {
        dsl_methods.push(build_public_dsl_method(method)?);
    }

    if let Some(method) = &input.spacetimedsl_methods.get_all {
        dsl_methods.push(build_public_dsl_method(method)?);
    }
    if let Some(method) = &input.spacetimedsl_methods.get_count {
        dsl_methods.push(build_public_dsl_method(method)?);
    }

    if let Some(strategies) = &input
        .spacetimedsl_methods
        .on_delete_strategies_of_referencing_tables
    {
        for entry_points in held_entry_points(&strategies.on_deletion, &strategies.on_soft_deletion)
        {
            dsl_methods.push(build_internal_dsl_method(&entry_points.after_one_row)?);
            dsl_methods.push(build_internal_dsl_method(
                &entry_points.after_multiple_rows,
            )?);
        }
    }

    // Two loops, not one: every one-row method is emitted before any many-row method, and a
    // single loop over the pairs would interleave them.
    let entry_points_of_this_table = || {
        input
            .spacetimedsl_methods
            .on_delete_strategies_of_this_table
            .iter()
            .flat_map(|strategies| {
                held_entry_points(&strategies.on_deletion, &strategies.on_soft_deletion)
            })
    };

    for entry_points in entry_points_of_this_table() {
        dsl_methods.push(build_internal_dsl_method(&entry_points.after_one_row)?);
    }

    for entry_points in entry_points_of_this_table() {
        dsl_methods.push(build_internal_dsl_method(
            &entry_points.after_multiple_rows,
        )?);
    }

    for multi_column_index in &input.spacetimedsl_methods.multi_column_indices {
        dsl_methods.extend(get_column_dsl_methods(multi_column_index)?);
    }

    for column in &input.columns {
        if pass.emits_the_struct_items() {
            if let Some(getter) = &column.spacetimedsl_column.getter {
                table_methods.push(accessor::build(accessor::Accessor::Getter(getter)));
            }

            if let Some(mut_getter) = &column.spacetimedsl_column.mut_getter {
                table_methods.push(accessor::build(accessor::Accessor::MutGetter(mut_getter)))
            }

            if let Some(setter) = &column.spacetimedsl_column.setter {
                table_methods.push(accessor::build(accessor::Accessor::Setter(setter)))
            }
        }

        if let Some(methods) = &column.spacetimedsl_methods {
            dsl_methods.extend(get_column_dsl_methods(methods)?)
        }
    }

    let mut compile_error_checks = vec![];

    input
        .spacetimedsl_table
        .compile_error_checks
        .iter()
        .for_each(|compile_error_check| {
            compile_error_checks.push(quote! {
                pub trait #compile_error_check {}
            });
        });

    let compile_error_check_imports = &input.spacetimedsl_table.compile_error_check_imports;

    // A block scope, so the imports of two tables in one module cannot clash.
    let compile_error_check_usages = match compile_error_check_imports.is_empty() {
        true => TokenStream::default(),
        false => quote! {
            const _: () = {
                #(use #compile_error_check_imports;)*
            };
        },
    };

    let create_dsl_method_arg = match &input.spacetimedsl_table.create_dsl_method_arg {
        Some(arg) => create_method_arg::build(&arg.struct_impl)?,
        None => TokenStream::default(),
    };

    let hooks: Vec<_> = input
        .spacetimedsl_table
        .hooks
        .iter()
        .map(hook::build)
        .collect();

    let referenced_row_methods: &[WrapperMethod] = match pass.emits_the_referenced_row_methods() {
        true => &input.spacetimedsl_methods.referenced_row_methods,
        false => &[],
    };

    let wrapper_methods = input
        .spacetimedsl_methods
        .wrapper_methods
        .iter()
        .chain(referenced_row_methods)
        .map(wrapper_method::build)
        .collect();

    Ok(GeneratedOutput {
        items_outside_dsl_methods: quote! {
            #(#compile_error_checks)*

            #compile_error_check_usages

            #(#wrapper_types)*

            impl #struct_name {
                #(#table_methods)*
            }

            #create_dsl_method_arg

            #(#hooks)*
        },
        dsl_methods,
        wrapper_methods,
    })
}

/// The cascade entry points one side of a foreign key relationship holds, those for deletion
/// before those for soft deletion.
fn held_entry_points<'a>(
    on_deletion: &'a Option<CascadeEntryPoints>,
    on_soft_deletion: &'a Option<CascadeEntryPoints>,
) -> impl Iterator<Item = &'a CascadeEntryPoints> {
    [on_deletion, on_soft_deletion].into_iter().flatten()
}

fn build_public_dsl_method(method: &SpacetimeDSLMethod) -> syn::Result<GeneratedDSLMethod> {
    Ok(GeneratedDSLMethod {
        method_name: method.method_name.clone(),
        is_internal: false,
        tokens: function::build_public(method)?,
    })
}

fn build_internal_dsl_method(method: &SpacetimeDSLMethod) -> syn::Result<GeneratedDSLMethod> {
    Ok(GeneratedDSLMethod {
        method_name: method.method_name.clone(),
        is_internal: true,
        tokens: function::build_internal(method)?,
    })
}

fn get_column_dsl_methods(
    methods: &SpacetimeDSLColumnMethods,
) -> syn::Result<Vec<GeneratedDSLMethod>> {
    let mut dsl_methods = vec![];

    match methods {
        SpacetimeDSLColumnMethods::ForUniqueIndex(methods) => {
            dsl_methods.push(build_public_dsl_method(&methods.get_one_option)?);

            if let Some(method) = &methods.update {
                dsl_methods.push(build_public_dsl_method(method)?)
            };

            if let Some(method) = &methods.delete_one {
                dsl_methods.push(build_public_dsl_method(method)?)
            };

            if let Some(method) = &methods.soft_delete_one {
                dsl_methods.push(build_public_dsl_method(method)?)
            };
        }
        SpacetimeDSLColumnMethods::ForIndex(methods) => {
            dsl_methods.push(build_public_dsl_method(&methods.get_many)?);

            if let Some(method) = &methods.delete_many {
                dsl_methods.push(build_public_dsl_method(method)?)
            };

            if let Some(method) = &methods.soft_delete_many {
                dsl_methods.push(build_public_dsl_method(method)?)
            };
        }
    };

    Ok(dsl_methods)
}

pub fn malformed_code_generation_result(result: String) -> String {
    let mut result = result.replace("\n", " ");

    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    format!("

Congratulations, you have found a bug in SpacetimeDSL!

We would be very pleased if you can create an issue in our GitHub repository: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/new

Please include your table definition as well as the following, malformed, code generation result - thank you very much!

{result}

")
}

fn map_args(args: &[SpacetimeDSLArg]) -> Vec<TokenStream> {
    args.iter()
        .map(|arg| {
            let arg_name = &arg.arg_name;
            let arg_type = arg.arg_type.actual_type();

            quote! {
                #arg_name: #arg_type
            }
        })
        .collect()
}
