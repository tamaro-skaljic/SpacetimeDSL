//! Covers a table whose accessor is `singleton`, named with `table = singleton`. Only the
//! `singleton` argument makes a table a singleton, not the word as the value of another
//! argument, so this table keeps its own primary key and gets no injected `id: u8`.

#[spacetimedsl::dsl(plural_name = singletons, table = singleton, method(update = false))]
#[spacetimedb::table(accessor = singleton, public)]
pub struct Singleton {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
}
