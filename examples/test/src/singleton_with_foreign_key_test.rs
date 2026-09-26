//! A `#[foreign_key]` column on a singleton table, for both singleton kinds.
//!
//! A singleton may carry a foreign key without an index, and the reference-integrity check
//! of its write path has to find the row by the injected primary key rather than through a
//! wrapper that key does not have. Only a module which is actually built proves that.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = regions, method(update = true))]
#[spacetimedb::table(
    accessor = region,
    public,
)]
pub struct Region {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(RegionId)]
    #[referenced_by(path = crate::singleton_with_foreign_key_test, table = server_binding)]
    #[referenced_by(path = crate::singleton_with_foreign_key_test, table = active_tournament)]
    id: u64,

    pub name: String,
}

#[spacetimedsl::dsl(singleton, method(update = true))]
#[spacetimedb::table(
    accessor = server_binding,
    public,
)]
pub struct ServerBinding {
    #[use_wrapper(RegionId)]
    #[foreign_key(path = crate::singleton_with_foreign_key_test, table = region, column = id, on_delete = Delete)]
    pub region_id: u64,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(
    accessor = active_tournament,
    public,
)]
pub struct ActiveTournament {
    #[use_wrapper(RegionId)]
    #[foreign_key(path = crate::singleton_with_foreign_key_test, table = region, column = id, on_delete = Delete)]
    pub region_id: u64,
}

impl DefaultSingleton for ActiveTournament {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<ActiveTournament, SpacetimeDSLError> {
        Ok(ActiveTournament {
            id: 0,
            // Zero means "no reference yet", which every reference-integrity check
            // skips, so a default is allowed to point nowhere.
            region_id: 0,
        })
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let region = dsl.create_region(CreateRegion {
        name: "Europe".to_string(),
    })?;

    // A singleton without a default: the reference-integrity check of `update_<table>`
    // has to find the row by the injected primary key.
    let mut server_binding = dsl.create_server_binding(CreateServerBinding {
        region_id: region.get_id(),
    })?;
    server_binding.set_region_id(region.get_id());

    dsl.update_server_binding(server_binding)
        .map_err(|e| format!("Updating a singleton with a foreign key should work! Got:\n{e}"))?;

    // A singleton with a default: its insert path and its update path each run their own
    // reference-integrity check.
    let mut tournament = dsl.get_active_tournament()?;
    tournament.set_region_id(region.get_id());

    let tournament = dsl.upsert_active_tournament(tournament).map_err(|e| {
        format!("Upserting a singleton with a foreign key should insert it! Got:\n{e}")
    })?;

    if tournament.get_region_id().ne(&region.get_id()) {
        return Err("The inserted region_id should be the one that was set!".to_string());
    }

    let mut tournament = tournament;
    tournament.set_region_id(RegionId::new(u64::MAX));

    dsl.upsert_active_tournament(tournament)
        .expect_err("Upserting a singleton whose foreign key points at no row should be rejected");

    let unchanged = dsl.get_active_tournament()?;
    if unchanged.get_region_id().ne(&region.get_id()) {
        return Err("A rejected upsert should leave the stored row alone!".to_string());
    }

    Ok(())
}
