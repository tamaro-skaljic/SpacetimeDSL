//! Covers a multi-column `index(accessor = ..., btree(columns = [...]))` that is *not*
//! declared unique, so the generated methods are the plural `get_many` / `delete_many`
//! pair instead of the single-row pair `unique_multi_column_index` pins.

#[spacetimedsl::dsl(plural_name = bookings, method(update = true))]
#[spacetimedb::table(
    accessor = booking,
    index(accessor = day_and_room, btree(columns = [day, room_id])),
    public,
)]
pub struct Booking {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub day: u32,

    pub room_id: u64,
}
