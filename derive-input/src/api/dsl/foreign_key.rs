use syn::{Ident, Path};

#[derive(Clone)]
pub struct ForeignKey {
    pub path: Path,
    pub table_name: Ident,
    pub primary_key_column_name: Ident,
    pub on_delete_strategy: OnDeleteStrategy,
}

// This enum is copy+paste of the enum in the SpacetimeDSL crate (which is the public API of the DSL).

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, strum::EnumIter)]
pub enum OnDeleteStrategy {
    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the deletion fails.
     */
    Error,

    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... it's checked whether any primary key value of rows to delete is referenced in a foreign key with `OnDeleteStrategy::Error`.
     * If true, the deletion fails and no other on delete strategy is executed.
     * If false, the on delete strategies of all affected rows are executed.
     */
    Delete,

    /**
     * Available only for tables with `#[dsl(method(soft_delete = true))]`.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the referencing rows are soft-deleted, which marks them through their marker column instead of removing them.
     * Like `Delete`, this cascades further into the tables which reference the soft-deleted rows.
     */
    SoftDelete,

    /**
     * TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 SetNone
     * Because Option is currently not allowed on primary_key and unique/btree indices this strategy isn't used and implemented yet.
     * Available only for columns with type `Option<T>`.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the value of the foreign key column is set to `None`.
     */
    //SetNone,

    /**
     * Available only for columns with a numeric type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... the value of the foreign key column is set to `0`.
     */
    SetZero,

    /**
     * Available independent from the column type.
     * If a row of a table should be deleted whose primary key value is referenced in foreign keys of other tables ...
     * ... nothing happens, which means the referencing rows will reference a primary key value which doesn't exist anymore.
     * The referential integrity is only enforced while creating a row or if a row is updated and the foreign key column value is changed.
     */
    Ignore,
}

impl quote::ToTokens for OnDeleteStrategy {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variant = match self {
            OnDeleteStrategy::Error => {
                crate::api::runtime::on_delete_strategy(&quote::quote! { Error })
            }
            OnDeleteStrategy::Delete => {
                crate::api::runtime::on_delete_strategy(&quote::quote! { Delete })
            }
            OnDeleteStrategy::SoftDelete => {
                crate::api::runtime::on_delete_strategy(&quote::quote! { SoftDelete })
            }
            OnDeleteStrategy::SetZero => {
                crate::api::runtime::on_delete_strategy(&quote::quote! { SetZero })
            }
            OnDeleteStrategy::Ignore => {
                crate::api::runtime::on_delete_strategy(&quote::quote! { Ignore })
            }
        };
        tokens.extend(variant);
    }
}
