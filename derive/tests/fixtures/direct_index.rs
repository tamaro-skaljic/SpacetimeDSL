//! Covers `#[index(direct)]` combined with `#[unique]` on the same column - the
//! combination that produced uncompilable code in
//! [#20](https://github.com/tamaro-skaljic/SpacetimeDSL/issues/20).

#[spacetimedsl::dsl(plural_name = slots, method(update = true))]
#[spacetimedb::table(accessor = slot, public)]
pub struct Slot {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(direct)]
    #[unique]
    pub position: u8,
}
