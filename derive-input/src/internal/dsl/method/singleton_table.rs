use {
    super::{context::MethodGenerationContext, message, removal, upsert},
    crate::{
        api::{
            dsl::{hook::HookKind, method::SpacetimeDSLMethod},
            runtime,
        },
        internal::dsl::{one_or_multiple::OneOrMultiple, singleton},
    },
    quote::{format_ident, quote},
};

/// `get_<table>`: the one row of a singleton table.
///
/// A singleton is found by its injected primary key rather than by an index the caller
/// supplies a value for, so this takes no arguments and renders no row values.
///
/// What happens while the row is absent is the whole difference between the two singleton
/// kinds: a table without a default fails, a table with one answers with the default its
/// struct supplies and leaves the table empty.
pub fn for_singleton_get(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    let primary_key = singleton::primary_key_ident();
    let primary_key_value = singleton::primary_key_value();

    let not_found_error = runtime::not_found_error(
        singular_table_name_as_string,
        &message::singleton_primary_key(),
    );

    let doc_comment = match spacetimedsl_table.singleton_has_default() {
        false => format!(
            "Try to get the `{struct_name}` from the singleton `{singular_table_name}` table."
        ),
        true => format!(
            "Get the `{struct_name}` from the singleton `{singular_table_name}` table, or its default while no row exists.\n\nThe default is not written to the table."
        ),
    };

    let absent_row = match spacetimedsl_table.singleton_has_default() {
        false => quote! {
            return Err(#not_found_error)
        },
        true => {
            let default_singleton_trait = runtime::default_singleton_trait();
            let read_only_dsl = runtime::read_only_dsl_call(&quote! { self.ctx() });
            let set_singleton_primary_key = upsert::set_singleton_primary_key(singular_table_name);

            quote! {
                {
                    let mut #singular_table_name =
                        <#struct_name as #default_singleton_trait>::get_default(&#read_only_dsl)?;
                    #set_singleton_primary_key
                    Ok(#singular_table_name)
                }
            }
        }
    };

    SpacetimeDSLMethod {
        doc_comment,
        method_name: format_ident!("get_{singular_table_name}"),
        method_args: vec![],
        return_type: runtime::error_result_type(struct_name),
        method_impl: quote! {
            match self.db().#singular_table_name().#primary_key().find(&#primary_key_value) {
                Some(#singular_table_name) => Ok(#singular_table_name),
                None => #absent_row
            }
        },
        // A read-only table handle offers primary key lookups.
        read_context_compatible: true,
    }
}

/// `delete_<table>`: delete the one row of a singleton table.
pub fn for_singleton_delete(context: &MethodGenerationContext) -> SpacetimeDSLMethod {
    let MethodGenerationContext {
        spacetimedsl_table,
        struct_name,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    let primary_key = singleton::primary_key_ident();
    let primary_key_value = singleton::primary_key_value();

    let before_delete_hook = removal::hard_deletion_hook(
        spacetimedsl_table,
        HookKind::BEFORE_DELETE,
        &OneOrMultiple::One,
    );

    let after_delete_hook = removal::hard_deletion_hook(
        spacetimedsl_table,
        HookKind::AFTER_DELETE,
        &OneOrMultiple::One,
    );

    let not_found_error = runtime::not_found_error(
        singular_table_name_as_string,
        &message::singleton_primary_key(),
    );

    let deletion_result_entry = runtime::deletion_result_entry(
        singular_table_name_as_string,
        &singleton::PRIMARY_KEY_NAME,
        &runtime::on_delete_strategy(&quote! { Delete }),
        &singleton::rendered_primary_key_value(),
        &quote! { vec![] },
    );

    let count_mismatch_error = removal::delete_one_count_mismatch_error();

    let single_entry_deletion_result = runtime::deletion_result(
        singular_table_name_as_string,
        &OneOrMultiple::One,
        &quote! { vec![deletion_result_entry] },
        &quote! { None },
    );

    SpacetimeDSLMethod {
        doc_comment: format!(
            "Try to delete the `{struct_name}` row from the singleton `{singular_table_name}` table."
        ),
        method_name: format_ident!("delete_{singular_table_name}"),
        method_args: vec![],
        return_type: runtime::error_result_type(&runtime::deletion_result_type()),
        method_impl: quote! {
            let row_to_delete = match self.db().#singular_table_name().#primary_key().find(&#primary_key_value) {
                None => return Err(#not_found_error),
                Some(row_to_delete) => row_to_delete,
            };

            let mut deletion_result_entry = #deletion_result_entry;

            #before_delete_hook

            match self.db().#singular_table_name().#primary_key().delete(&#primary_key_value) {
                false => {
                    return Err(#count_mismatch_error);
                },
                true => {},
            };

            #after_delete_hook

            return Ok(#single_entry_deletion_result);
        },
        // Deleting writes.
        read_context_compatible: false,
    }
}
