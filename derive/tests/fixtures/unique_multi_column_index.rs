//! Covers `#[dsl(unique_index(name = ...))]` over an `#[index(accessor = ..., btree(columns = [...]))]`:
//! the experimental path that also emits the uniqueness compile-error check and the
//! experimental-feature warning `IndexShape::of` attaches to a unique multi-column index.

#[spacetimedsl::dsl(
    plural_name = seats,
    method(update = true),
    unique_index(name = row_and_number),
)]
#[spacetimedb::table(
    accessor = seat,
    index(accessor = row_and_number, btree(columns = [row_id, number])),
    public,
)]
pub struct Seat {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub row_id: u64,

    pub number: u32,
}
