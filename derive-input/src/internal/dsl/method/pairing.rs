//! The compile-time checks which pair a table with the tables on the other side of its foreign
//! keys, and the facts every table states for the checks of the others.
//!
//! A table cannot see how the tables on the other side are declared, only what their expansion
//! emits. So every accepted table implements the fact traits of `crate::spacetimedsl::internal`:
//! `AcceptedTable` for its `<accessor>__TableHandle`, the others for its table type. Each check
//! is a trait of its own, whose `#[diagnostic::on_unimplemented]` says what to change with both
//! table names filled in. A `#[diagnostic::do_not_recommend]` blanket impl implements it for
//! every implementer of one fact, so the compiler reports the check rather than the fact.
//!
//! A rejected table keeps its handle but has no table type. Its gate, the check of its handle,
//! fails, and every other check of it names a missing item, which the compiler leaves unreported
//! in a module with a `compile_error!`. So an attribute which depends on a rejected table reports
//! one error.

use {
    super::{
        context::{MethodGenerationContext, TableContributions},
        foreign_key::foreign_key_of,
        naming::{self, cascade_binding},
        removal::{Removal, dispatcher_signature},
    },
    crate::{
        api::{Column, dsl::reference::ReferencingTable, runtime},
        internal::{dsl::one_or_multiple::OneOrMultiple, spacetimedb},
    },
    itertools::Itertools,
    proc_macro2::{Group, Span, TokenStream, TokenTree},
    quote::{format_ident, quote},
    std::collections::BTreeMap,
    syn::{Ident, Path},
};

/// `<path::<Referencing>Table as <Referencing>TableReferencesThe<This>Table>::#item`, the only way
/// from the cascades of `this_table` into the table `referencing_table` names. Spanned on the
/// `table = …` of that `#[referenced_by]`, like the check that it has a foreign key back, so
/// that a missing one fails once.
pub fn cascade_into(
    referencing_table: &ReferencingTable,
    this_table: &Ident,
    item: &Ident,
) -> TokenStream {
    let referencing_table_name = &referencing_table.table_name;
    let referencing_table_type = table_type_path(&referencing_table.path, referencing_table_name);
    let check_name = naming::foreign_key_back_check_name(referencing_table_name, this_table);

    respan(
        quote! { <#referencing_table_type as #check_name>::#item },
        referencing_table_name.span(),
    )
}

/// The facts the table of `context` states about itself, the checks it makes of the tables on
/// the other side of its foreign keys and `#[referenced_by]` attributes, and the checks
/// through which its cascades reach the tables referencing it.
pub fn for_table(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let facts = facts(context);

    let mut checks = checks_of_referenced_tables(context, foreign_key_columns_by_referenced_table);

    checks.extend(gates_of_referencing_tables(context));

    // A table without foreign keys and `#[referenced_by]` makes no checks.
    let checks = (!checks.is_empty()).then(|| {
        quote! {
            const _: () = {
                #(#checks)*
            };
        }
    });

    let foreign_key_back_checks = foreign_key_back_checks(context);

    TableContributions {
        pairing: quote! {
            #facts

            #checks

            #(#foreign_key_back_checks)*
        },
        ..TableContributions::default()
    }
}

/// `AcceptedTable` for the table's handle, which removals the table performs, and which tables
/// its `#[referenced_by]` attributes name, for its table type.
fn facts(context: &MethodGenerationContext) -> TokenStream {
    let MethodGenerationContext {
        spacetimedsl_table,
        singular_table_name,
        ..
    } = context;

    let accepted_table = runtime::accepted_table_trait();
    let table_handle = spacetimedb::table_handle_name(singular_table_name);
    let table_type = naming::table_type_name(singular_table_name);

    let removal_facts = Removal::ALL.into_iter().map(|removal| {
        let fact = removal_fact(removal, removal.is_performed_by(spacetimedsl_table));

        quote! {
            impl #fact for #table_type {}
        }
    });

    let referenced_by_facts =
        spacetimedsl_table
            .referencing_tables
            .iter()
            .map(|referencing_table| {
                let referenced_by = runtime::referenced_by_trait(&table_type_path(
                    &referencing_table.path,
                    &referencing_table.table_name,
                ));

                quote! {
                    impl #referenced_by for #table_type {}
                }
            });

    quote! {
        impl #accepted_table for #table_handle {}

        #(#removal_facts)*

        #(#referenced_by_facts)*
    }
}

/// The fact which says whether a table performs `removal`.
fn removal_fact(removal: Removal, is_performed: bool) -> TokenStream {
    match (removal, is_performed) {
        (Removal::Hard, true) => runtime::has_delete_method_trait(),
        (Removal::Hard, false) => runtime::has_no_delete_method_trait(),
        (Removal::Soft, true) => runtime::is_soft_deletable_trait(),
        (Removal::Soft, false) => runtime::is_not_soft_deletable_trait(),
    }
}

/// For each foreign key: that the referenced table is accepted, that it names this table in
/// `#[referenced_by]`, and per removal that the foreign key declares a strategy exactly when the
/// referenced table performs the removal.
fn checks_of_referenced_tables(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> Vec<TokenStream> {
    let this_table = &context.singular_table_name;
    let this_table_type = naming::table_type_name(this_table);

    let mut checks = vec![];

    for (referenced_table, columns_with_foreign_key) in foreign_key_columns_by_referenced_table {
        for column in columns_with_foreign_key {
            let foreign_key = foreign_key_of(column);
            let span = foreign_key.table_name.span();
            let referenced_table_type = table_type_path(&foreign_key.path, referenced_table);

            checks.push(check(
                span,
                &format_ident!("{}IsAccepted", naming::table_type_name(referenced_table)),
                &runtime::accepted_table_trait(),
                &spacetimedb::table_handle_type(&foreign_key.path, referenced_table),
                words::referenced_table_is_accepted(referenced_table),
            ));

            checks.push(check(
                span,
                &format_ident!("ReferencedByThe{}", this_table_type),
                &runtime::referenced_by_trait(&this_table_type),
                &referenced_table_type,
                words::referenced_table_names_this_table(referenced_table, this_table),
            ));

            for removal in Removal::ALL {
                let declares_a_strategy = removal.strategy_declared_by(foreign_key).is_some();
                let fact = removal_fact(removal, declares_a_strategy);
                let check_name = removal_check_name(referenced_table, removal, declares_a_strategy);

                checks.push(check(
                    span,
                    &check_name,
                    &fact,
                    &referenced_table_type,
                    match declares_a_strategy {
                        true => words::strategy_can_apply(removal, referenced_table),
                        false => words::strategy_is_set(removal, referenced_table),
                    },
                ));
            }
        }
    }

    checks
}

/// The name of the check that `referenced_table` states `removal_fact`, such as
/// `WarehouseTableHasADeleteMethod`, which the compiler shows when the fact is missing.
fn removal_check_name(referenced_table: &Ident, removal: Removal, is_performed: bool) -> Ident {
    let fact = match (removal, is_performed) {
        (Removal::Hard, true) => "HasADeleteMethod",
        (Removal::Hard, false) => "HasNoDeleteMethod",
        (Removal::Soft, true) => "IsSoftDeletable",
        (Removal::Soft, false) => "IsNotSoftDeletable",
    };

    format_ident!("{}{fact}", naming::table_type_name(referenced_table))
}

/// For each table the `#[referenced_by]` attributes name: that it is accepted.
fn gates_of_referencing_tables(context: &MethodGenerationContext) -> Vec<TokenStream> {
    let this_table = &context.singular_table_name;

    context
        .spacetimedsl_table
        .referencing_tables
        .iter()
        .map(|referencing_table| {
            let referencing_table_name = &referencing_table.table_name;

            check(
                referencing_table_name.span(),
                &format_ident!(
                    "{}IsAccepted",
                    naming::table_type_name(referencing_table_name)
                ),
                &runtime::accepted_table_trait(),
                &spacetimedb::table_handle_type(&referencing_table.path, referencing_table_name),
                words::referencing_table_is_accepted(referencing_table_name, this_table),
            )
        })
        .collect()
}

/// For each table the `#[referenced_by]` attributes name: the check that it has a foreign key
/// back, a trait which also carries the cascades of `internal::References` and implements them
/// for every implementer of it.
///
/// The cascades of this table call into the referencing table only through it, spanned like the
/// check itself, on the `table = …` of the `#[referenced_by]`. A missing foreign key fails every
/// one of them with the same error at the same place, which the compiler reports once.
fn foreign_key_back_checks(context: &MethodGenerationContext) -> Vec<TokenStream> {
    let this_table = &context.singular_table_name;
    let primary_key_type = &context.primary_key_column.rust_field_type_name_or_path;
    let references =
        runtime::references_trait(&naming::table_type_name(this_table), primary_key_type);
    let write_context = runtime::write_context();

    let cascades = Removal::ALL.into_iter().flat_map(|removal| {
        [OneOrMultiple::One, OneOrMultiple::Multiple].map(|one_or_multiple| {
            let cascade_method_name = naming::cascade_method_name(removal, &one_or_multiple);
            let (cascade_args, return_type) = dispatcher_signature(
                &one_or_multiple,
                {
                    let on_delete_strategy_type = runtime::on_delete_strategy_type();

                    quote! { &#on_delete_strategy_type }
                },
                &match one_or_multiple {
                    OneOrMultiple::One => {
                        cascade_binding::primary_key_value_of_a_row_of_another_table_to_delete()
                    }
                    OneOrMultiple::Multiple => {
                        cascade_binding::primary_key_values_of_rows_of_another_table_to_delete()
                    }
                },
                primary_key_type,
            );
            let arg_names = cascade_args.iter().map(|arg| &arg.arg_name).collect_vec();
            let arg_types = cascade_args.iter().map(|arg| arg.arg_type.actual_type());
            let signature = quote! {
                fn #cascade_method_name<'a, T: #write_context>(
                    &self,
                    #(#arg_names: #arg_types),*
                ) -> #return_type
            };

            (
                quote! { #signature; },
                quote! {
                    #signature {
                        <Referencing as #references>::#cascade_method_name(self, #(#arg_names),*)
                    }
                },
            )
        })
    });
    let (cascade_declarations, cascade_implementations): (Vec<_>, Vec<_>) = cascades.unzip();

    context
        .spacetimedsl_table
        .referencing_tables
        .iter()
        .map(|referencing_table| {
            let referencing_table_name = &referencing_table.table_name;
            let check_name =
                naming::foreign_key_back_check_name(referencing_table_name, this_table);
            let words::Words { message, label, .. } =
                words::referencing_table_has_a_foreign_key_back(referencing_table_name, this_table);
            let holds = cascade_into(referencing_table, this_table, &format_ident!("HOLDS"));

            quote! {
                // The cascades take a key by reference, as `internal::References` does, also a
                // `&String`.
                #[diagnostic::on_unimplemented(message = #message, label = #label)]
                #[allow(dead_code, clippy::ptr_arg)]
                trait #check_name {
                    const HOLDS: () = ();

                    #(#cascade_declarations)*
                }

                #[diagnostic::do_not_recommend]
                impl<Referencing: #references> #check_name for Referencing {
                    #(#cascade_implementations)*
                }

                const _: () = #holds;
            }
        })
        .collect()
}

/// `const _: () = <#subject as #check_name>::HOLDS;` with the trait `check_name` and its blanket
/// impl over `fact`, in a block of its own. The line which fails is spanned on `span`.
fn check(
    span: Span,
    check_name: &Ident,
    fact: &TokenStream,
    subject: &TokenStream,
    words: words::Words,
) -> TokenStream {
    let words::Words {
        message,
        label,
        note,
    } = words;
    let note = note.map(|note| quote! { , note = #note });
    let holds = respan(quote! { <#subject as #check_name>::HOLDS }, span);

    quote! {
        const _: () = {
            #[diagnostic::on_unimplemented(message = #message, label = #label #note)]
            trait #check_name {
                const HOLDS: () = ();
            }

            #[diagnostic::do_not_recommend]
            impl<T: #fact> #check_name for T {}

            const _: () = #holds;
        };
    }
}

/// `<module_path>::<Table>Table`, the table type of the table `table_name` in `module_path`.
fn table_type_path(module_path: &Path, table_name: &Ident) -> TokenStream {
    let table_type = naming::table_type_name(table_name);

    quote! { #module_path::#table_type }
}

/// `tokens` with every token spanned on `span`, so that the compiler underlines the attribute the
/// check belongs to rather than `#[dsl]`.
fn respan(tokens: TokenStream, span: Span) -> TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            TokenTree::Group(group) => {
                let mut respanned = Group::new(group.delimiter(), respan(group.stream(), span));
                respanned.set_span(span);
                TokenTree::Group(respanned)
            }
            mut token => {
                token.set_span(span);
                token
            }
        })
        .collect()
}

/// What each check says when its fact is missing.
mod words {
    use {super::Removal, syn::Ident};

    /// The `message`, `label` and `note` of `#[diagnostic::on_unimplemented]`.
    pub struct Words {
        pub message: String,
        pub label: String,
        pub note: Option<String>,
    }

    pub fn referenced_table_is_accepted(referenced_table: &Ident) -> Words {
        Words {
            message: format!(
                "the `{referenced_table}` table can't be referenced, because it has no accepted `#[spacetimedsl::dsl]`"
            ),
            label: format!(
                "fix the `#[spacetimedsl::dsl]` of the `{referenced_table}` table first, or add one"
            ),
            note: None,
        }
    }

    pub fn referencing_table_is_accepted(
        referencing_table: &Ident,
        referenced_table: &Ident,
    ) -> Words {
        Words {
            message: format!(
                "the `{referencing_table}` table can't reference the `{referenced_table}` table, because the `{referencing_table}` table has no accepted `#[spacetimedsl::dsl]`"
            ),
            label: format!(
                "fix the `#[spacetimedsl::dsl]` of the `{referencing_table}` table first, or add one"
            ),
            note: None,
        }
    }

    pub fn referenced_table_names_this_table(
        referenced_table: &Ident,
        this_table: &Ident,
    ) -> Words {
        Words {
            message: format!(
                "the `{referenced_table}` table has no `#[referenced_by]` naming the `{this_table}` table"
            ),
            label: format!("the `{referenced_table}` table doesn't know about this foreign key"),
            note: Some(format!(
                "add `#[referenced_by(path = …, table = {this_table})]` to the primary key column of the `{referenced_table}` table"
            )),
        }
    }

    pub fn strategy_can_apply(removal: Removal, referenced_table: &Ident) -> Words {
        let strategy_argument = removal.strategy_argument();
        let has_no_removal = match removal {
            Removal::Hard => "has no delete method",
            Removal::Soft => "isn't soft-deletable",
        };

        Words {
            message: format!(
                "the `{referenced_table}` table {has_no_removal}, so `{strategy_argument}` can't apply"
            ),
            label: format!("remove `{strategy_argument}`"),
            note: None,
        }
    }

    pub fn strategy_is_set(removal: Removal, referenced_table: &Ident) -> Words {
        let strategy_argument = removal.strategy_argument();
        let (has_removal, strategies) = match removal {
            Removal::Hard => (
                "has a delete method",
                "`Error`, `Delete`, `SoftDelete`, `SetZero` or `Ignore`",
            ),
            Removal::Soft => ("is soft-deletable", "`Error`, `SoftDelete` or `Ignore`"),
        };

        Words {
            message: format!(
                "the `{referenced_table}` table {has_removal}, so this foreign key needs `{strategy_argument}`"
            ),
            label: format!("add `{strategy_argument} = …`"),
            note: Some(format!("`{strategy_argument}` takes {strategies}")),
        }
    }

    pub fn referencing_table_has_a_foreign_key_back(
        referencing_table: &Ident,
        referenced_table: &Ident,
    ) -> Words {
        Words {
            message: format!(
                "the `{referencing_table}` table has no `#[foreign_key]` referencing the `{referenced_table}` table"
            ),
            label: format!(
                "remove this `#[referenced_by]`, or add the foreign key to the `{referencing_table}` table"
            ),
            note: None,
        }
    }
}
