//! Covers `#[create_wrapper]` on `Uuid` columns, in the bare and the `spacetimedb::` spelling.
//! `Uuid` has no `Default`, so its wrapper has no `Default` either and offers `v4` and `v7`
//! to generate a fresh value instead.

use spacetimedb::Uuid;

#[spacetimedsl::dsl(plural_name = licenses, method(update = true))]
#[spacetimedb::table(accessor = license, public)]
pub struct License {
    #[primary_key]
    #[create_wrapper]
    id: Uuid,

    #[unique]
    #[create_wrapper(ExternalReference)]
    pub external_reference: spacetimedb::Uuid,
}
