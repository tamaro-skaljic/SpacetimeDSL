//! Covers `#[create_wrapper(Name)]`, the explicitly named counterpart of
//! `wrapper_created_unnamed`. The generated wrapper type and every method signature
//! mentioning it must use the given name instead of the derived one.

#[spacetimedsl::dsl(plural_name = invoices, method(update = true))]
#[spacetimedb::table(accessor = invoice, public)]
pub struct Invoice {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(InvoiceNumber)]
    id: u64,

    #[unique]
    #[create_wrapper(InvoiceReference)]
    pub reference: String,
}
