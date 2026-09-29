//! Covers a table whose rows are never removed, `method(delete = false)` without
//! `method(soft_delete = true)`, and a foreign key to it. The referenced table offers no
//! cascade entry point and declares the traits which let a foreign key without strategies
//! compile; the referencing table imports them and generates no strategy implementation.

#[spacetimedsl::dsl(plural_name = currencies, method(update = false, delete = false))]
#[spacetimedb::table(accessor = currency, public)]
pub struct Currency {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(CurrencyId)]
    #[referenced_by(path = self, table = price)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = prices, method(update = true, delete = true))]
#[spacetimedb::table(accessor = price, public)]
pub struct Price {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CurrencyId)]
    #[foreign_key(path = self, table = currency, column = id)]
    pub currency_id: u64,
}
