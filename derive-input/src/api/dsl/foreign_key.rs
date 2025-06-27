#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub struct ForeignKey {
    pub path: Box<str>,
    pub table_name: Box<str>,
    pub on_delete_strategy: OnDeleteStrategy,
}

// This enum is copy+paste of the enum in the SpacetimeDSL crate (which is the public API of the DSL).

#[derive(Clone, Debug, PartialEq, PartialOrd, spacetimedb::SpacetimeType, Hash, Eq, Ord)]
pub enum OnDeleteStrategy {
    /// Available independent from the column type.
    Error,

    /// Available independent from the column type.
    Delete,

    // TODO: Because Option is currently not allowed on primary_key and unique/btree indices this strategy isn't used and implemented yet.
    /// Available only for columns with type `Option<T>`.
    //SetNone,

    /// Available only for columns with a numeric type.
    SetZero,
}

impl quote::ToTokens for OnDeleteStrategy {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use proc_macro2::{Punct, Spacing};
        use quote::{TokenStreamExt, format_ident};

        tokens.append(format_ident!("spacetimedsl"));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(format_ident!("OnDeleteStrategy"));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(format_ident!(
            "{}",
            match self {
                OnDeleteStrategy::Error => "Error",
                OnDeleteStrategy::Delete => "Delete",
                OnDeleteStrategy::SetZero => "SetZero",
            },
        ));
    }
}
