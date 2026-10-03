//! Covers `#[creation_default(...)]` on the column shapes a caller would otherwise supply: a
//! plain public column, a private one, a `#[create_wrapper]` column, a `#[use_wrapper]`
//! foreign key column and an `Option`. Each is left out of `CreateLoan`, filled with its
//! expression while `create_loan` builds the row, and listed under *Defaults* in its
//! documentation, which writes the expression without the spaces `proc_macro2` puts
//! between its tokens.
//!
//! The fixture is only expanded, never compiled, so `LoanState` does not have to exist.

#[spacetimedsl::dsl(plural_name = branches, method(update = false, delete = false))]
#[spacetimedb::table(accessor = branch, public)]
pub struct Branch {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = loan)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = loans, method(update = true))]
#[spacetimedb::table(accessor = loan, public)]
pub struct Loan {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub borrower: String,

    #[creation_default(-1)]
    pub priority: i32,

    #[creation_default(LoanState::Requested)]
    state: LoanState,

    #[index(btree)]
    #[create_wrapper]
    #[creation_default(String::from("general purpose"))]
    pub purpose: String,

    #[index(btree)]
    #[use_wrapper(BranchId)]
    #[foreign_key(path = self, table = branch, column = id)]
    #[creation_default(0)]
    pub branch_id: u64,

    #[creation_default(vec![1, 2])]
    pub reminder_days: Vec<u8>,

    #[creation_default(None)]
    pub note: Option<String>,
}
