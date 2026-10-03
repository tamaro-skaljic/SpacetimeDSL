//! `#[disallow(zero)]`: a column whose value must not be `0`, or `Uuid::NIL` for a `Uuid`.
//!
//! `create_<table>`, `update_<table>_by_<key>` and both paths of `upsert_<table>` refuse a row
//! which breaks the rule, after their before hook, so a hook may repair the value.
//! `create_<table>` skips an `#[auto_inc]` column, whose placeholder `0` SpacetimeDB replaces.

use {crate::spacetimedsl::prelude::*, spacetimedb::Uuid};

#[spacetimedsl::dsl(plural_name = disallow_zero_items, method(update = true))]
#[spacetimedb::table(accessor = disallow_zero_item)]
pub struct DisallowZeroItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[disallow(zero)]
    id: u64,

    #[disallow(zero)]
    pub stock: u32,

    #[disallow(zero)]
    pub serial: Uuid,
}

/// An item whose hooks turn a `stock` of `0` into `1` before the rule is checked.
#[spacetimedsl::dsl(
    plural_name = disallow_zero_repaired_items,
    method(update = true),
    hook(before(insert, update))
)]
#[spacetimedb::table(accessor = disallow_zero_repaired_item)]
pub struct DisallowZeroRepairedItem {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[disallow(zero)]
    pub stock: u32,
}

#[spacetimedsl::hook]
fn before_disallow_zero_repaired_item_insert(
    _dsl: &DSL<'_, T>,
    mut item: CreateDisallowZeroRepairedItem,
) -> Result<CreateDisallowZeroRepairedItem, SpacetimeDSLError> {
    if item.stock == 0 {
        item.stock = 1;
    }

    Ok(item)
}

#[spacetimedsl::hook]
fn before_disallow_zero_repaired_item_update(
    _dsl: &DSL<'_, T>,
    _old_item: &DisallowZeroRepairedItem,
    mut new_item: DisallowZeroRepairedItem,
) -> Result<DisallowZeroRepairedItem, SpacetimeDSLError> {
    if *new_item.get_stock() == 0 {
        new_item.set_stock(1);
    }

    Ok(new_item)
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true))]
#[spacetimedb::table(accessor = disallow_zero_limits)]
pub struct DisallowZeroLimits {
    #[disallow(zero)]
    pub maximum_stock: u32,
}

impl DefaultSingleton for DisallowZeroLimits {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<DisallowZeroLimits, SpacetimeDSLError> {
        Ok(DisallowZeroLimits {
            id: 0,
            maximum_stock: 100,
        })
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    create_skips_the_auto_inc_primary_key(dsl)?;
    create_refuses_a_zero(dsl)?;
    create_refuses_a_nil_uuid(dsl)?;
    update_refuses_a_zero(dsl)?;
    a_before_hook_can_repair_the_value(dsl)?;
    upsert_refuses_a_zero_on_both_paths(dsl)?;

    Ok(())
}

/// `id` is `#[auto_inc]` and `#[disallow(zero)]`: create writes the placeholder `0` into it,
/// which SpacetimeDB replaces, so the rule must not refuse the row.
fn create_skips_the_auto_inc_primary_key<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let item = dsl
        .create_disallow_zero_item(CreateDisallowZeroItem {
            stock: 5,
            serial: dsl.ctx().new_uuid_v4()?,
        })
        .map_err(|error| {
            format!(
                "Should create an item whose `#[auto_inc]` key has `#[disallow(zero)]`! Got:\n{error}"
            )
        })?;

    if item.get_id().value() == 0 {
        return Err(
            "SpacetimeDB should have replaced the placeholder 0 of the `#[auto_inc]` key!"
                .to_string(),
        );
    }

    Ok(())
}

fn create_refuses_a_zero<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let creation = dsl.create_disallow_zero_item(CreateDisallowZeroItem {
        stock: 0,
        serial: dsl.ctx().new_uuid_v4()?,
    });

    expect_disallowed(
        creation,
        "Disallowed Value Error while trying to create a row in the `disallow_zero_item` table because `stock` is `0`, which `#[disallow(zero)]` forbids!",
    )
}

fn create_refuses_a_nil_uuid<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let creation = dsl.create_disallow_zero_item(CreateDisallowZeroItem {
        stock: 5,
        serial: Uuid::NIL,
    });

    expect_disallowed(
        creation,
        "Disallowed Value Error while trying to create a row in the `disallow_zero_item` table because `serial` is `Uuid::NIL`, which `#[disallow(zero)]` forbids!",
    )
}

fn update_refuses_a_zero<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut item = dsl.create_disallow_zero_item(CreateDisallowZeroItem {
        stock: 5,
        serial: dsl.ctx().new_uuid_v4()?,
    })?;
    let expected_message = format!(
        "Disallowed Value Error while trying to update the row `{{ id : {} }}` in the `disallow_zero_item` table because `stock` is `0`, which `#[disallow(zero)]` forbids!",
        item.get_id().value()
    );
    item.set_stock(0);

    expect_disallowed(dsl.update_disallow_zero_item_by_id(item), &expected_message)
}

/// The rule is checked after the before hooks, which turn a `stock` of `0` into `1`.
fn a_before_hook_can_repair_the_value<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut item = dsl
        .create_disallow_zero_repaired_item(CreateDisallowZeroRepairedItem { stock: 0 })
        .map_err(|error| {
            format!("The before_insert hook should have repaired the stock of 0! Got:\n{error}")
        })?;

    if *item.get_stock() != 1 {
        return Err(format!(
            "The before_insert hook should have written a stock of 1! Got: {}",
            item.get_stock()
        ));
    }

    item.set_stock(0);

    let item = dsl
        .update_disallow_zero_repaired_item_by_id(item)
        .map_err(|error| {
            format!("The before_update hook should have repaired the stock of 0! Got:\n{error}")
        })?;

    if *item.get_stock() != 1 {
        return Err(format!(
            "The before_update hook should have written a stock of 1! Got: {}",
            item.get_stock()
        ));
    }

    Ok(())
}

/// The insert path of `upsert_disallow_zero_limits` checks the row it is about to create, the
/// update path the row it is about to write over the stored one.
fn upsert_refuses_a_zero_on_both_paths<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut limits = dsl.get_disallow_zero_limits()?;
    limits.set_maximum_stock(0);

    expect_disallowed(
        dsl.upsert_disallow_zero_limits(limits.clone()),
        "Disallowed Value Error while trying to create a row in the `disallow_zero_limits` table because `maximum_stock` is `0`, which `#[disallow(zero)]` forbids!",
    )?;

    limits.set_maximum_stock(50);
    let mut limits = dsl.upsert_disallow_zero_limits(limits)?;
    limits.set_maximum_stock(0);

    expect_disallowed(
        dsl.upsert_disallow_zero_limits(limits),
        "Disallowed Value Error while trying to update the row `{ id : 0 }` in the `disallow_zero_limits` table because `maximum_stock` is `0`, which `#[disallow(zero)]` forbids!",
    )
}

/// Checks that `write` failed with the message of a broken `#[disallow]` rule. The groups of
/// the other rules check their writes with it too.
pub(crate) fn expect_disallowed<Row: std::fmt::Debug>(
    write: Result<Row, SpacetimeDSLError>,
    expected_message: &str,
) -> Result<(), String> {
    match write {
        Err(SpacetimeDSLError::Error(message)) if message == expected_message => Ok(()),
        other => Err(format!(
            "The write should fail with \"{expected_message}\"! Got: {other:?}"
        )),
    }
}
