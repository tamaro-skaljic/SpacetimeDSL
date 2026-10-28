//! A table whose only update hook runs after the write. The hook receives the row as it was
//! stored before the update, although no before-update hook, foreign key check or `#[disallow]`
//! rule looks that row up first.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(
    plural_name = thermostats,
    method(update = true),
    hook(after(update))
)]
#[spacetimedb::table(accessor = thermostat)]
pub struct Thermostat {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    pub target_temperature: u32,
}

/// One row per call of `after_thermostat_update`, with the temperatures of the two rows it
/// received.
#[spacetimedsl::dsl(plural_name = thermostat_changes, method(update = false))]
#[spacetimedb::table(accessor = thermostat_change)]
pub struct ThermostatChange {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    thermostat_id: u64,

    old_target_temperature: u32,

    new_target_temperature: u32,
}

#[spacetimedsl::hook]
fn after_thermostat_update(
    dsl: &DSL<'_, T>,
    old_thermostat: &Thermostat,
    new_thermostat: &Thermostat,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_thermostat_change(CreateThermostatChange {
        thermostat_id: new_thermostat.get_id().value(),
        old_target_temperature: *old_thermostat.get_target_temperature(),
        new_target_temperature: *new_thermostat.get_target_temperature(),
    })?;

    Ok(())
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut thermostat = dsl.create_thermostat(CreateThermostat {
        target_temperature: 20,
    })?;
    thermostat.set_target_temperature(22);

    let thermostat = dsl
        .update_thermostat_by_id(thermostat)
        .map_err(|error| format!("Updating a thermostat should succeed: {error}"))?;

    let changes: Vec<(u32, u32)> = dsl
        .get_all_thermostat_changes()
        .filter(|change| *change.get_thermostat_id() == thermostat.get_id().value())
        .map(|change| {
            (
                *change.get_old_target_temperature(),
                *change.get_new_target_temperature(),
            )
        })
        .collect();

    if changes != [(20, 22)] {
        return Err(format!(
            "The after-update hook should run once, with the stored row of 20 and the written row of 22, found {changes:?}!"
        ));
    }

    Ok(())
}
