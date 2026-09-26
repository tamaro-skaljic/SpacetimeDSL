use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(
    plural_name = attributes,
    method(update = true),
    hook(
        before(
            insert,
            update,
            delete,
        ),
        after(
            insert,
            update,
            delete,
        ),
    ),
)]
#[spacetimedb::table(
    accessor = attribute,
    public,
)]
pub struct Attribute {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = potion)]
    id: u128,

    pub value: String,
}

#[spacetimedsl::dsl(
    plural_name = potions,
    method(update = true),
    hook(
        before(
            insert,
            update,
            delete,
        ),
        after(
            insert,
            update,
            delete,
        ),
    ),
)]
#[spacetimedb::table(
    accessor = potion,
    public,
)]
pub struct Potion {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = recipe)]
    id: u128,

    #[unique]
    pub value: String,

    #[index(btree)]
    #[use_wrapper(AttributeId)]
    #[foreign_key(path = self, table = attribute, column = id, on_delete = Delete)]
    attribute_id: u128,

    #[unique]
    #[use_wrapper(AttributeId)]
    #[foreign_key(path = self, table = attribute, column = id, on_delete = Delete)]
    unique_attribute_id: u128,
}

#[spacetimedsl::dsl(
    plural_name = recipes,
    method(update = true),
    hook(
        before(
            insert,
            update,
            delete,
        ),
        after(
            insert,
            update,
            delete,
        ),
    ),
)]
#[spacetimedb::table(
    accessor = recipe,
    public,
)]
pub struct Recipe {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    #[unique]
    pub value: String,

    #[index(btree)]
    #[use_wrapper(PotionId)]
    #[foreign_key(path = self, table = potion, column = id, on_delete = Delete)]
    potion_id: u128,
}

#[spacetimedsl::dsl(
    plural_name = multi_column_index_with_hook_tests,
    method(update = true),
    hook(
        before(
            insert,
            update,
            delete,
        ),
        after(
            insert,
            update,
            delete,
        ),
    ),
)]
#[spacetimedb::table(
    accessor = multi_column_index_with_hook_test,
    index(
        accessor = value_1_and_2,
        btree(columns = [value_1, value_2])
    ),
    public,
)]
pub struct MultiColumnIndexWithHookTest {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    pub value_1: bool,

    pub value_2: bool,
}

#[spacetimedsl::dsl(
    plural_name = hook_calls,
    method(
        update = false,
        delete = false,
    ),
)]
#[spacetimedb::table(
    accessor = hook_call,
    public,
)]
pub struct HookCall {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u128,

    value: String,

    created_at: Timestamp,
}

#[spacetimedsl::hook]
fn before_attribute_insert(
    dsl: &DSL<'_, T>,
    mut create_attribute_request: CreateAttribute,
) -> Result<CreateAttribute, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_ATTRIBUTE_INSERT".to_string(),
    })?;

    create_attribute_request.value = format!("{}_ATTRIBUTE", create_attribute_request.value);

    Ok(create_attribute_request)
}

#[spacetimedsl::hook]
fn after_attribute_insert(
    dsl: &DSL<'_, T>,
    new_attribute: &Attribute,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_ATTRIBUTE_INSERT".to_string(),
    })?;

    dsl.create_potion(CreatePotion {
        value: format!("PERMANENT_{}_INCREASE", new_attribute.value),
        attribute_id: new_attribute.get_id(),
        unique_attribute_id: new_attribute.get_id(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_attribute_update(
    dsl: &DSL<'_, T>,
    _old_attribute: &Attribute,
    mut new_attribute: Attribute,
) -> Result<Attribute, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_ATTRIBUTE_UPDATE".to_string(),
    })?;

    new_attribute.set_value(format!("{}_ATTRIBUTE", new_attribute.get_value()));

    Ok(new_attribute)
}

#[spacetimedsl::hook]
fn after_attribute_update(
    dsl: &DSL<'_, T>,
    old_attribute: &Attribute,
    new_attribute: &Attribute,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_ATTRIBUTE_UPDATE".to_string(),
    })?;

    let mut potion = dsl.get_potion_by_value(&format!(
        "PERMANENT_{}_INCREASE_POTION",
        old_attribute.get_value()
    ))?;

    potion.set_value(format!("PERMANENT_{}_INCREASE", new_attribute.get_value()));

    dsl.update_potion_by_id(potion)?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_attribute_delete(
    dsl: &DSL<'_, T>,
    _old_attribute: &Attribute,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_ATTRIBUTE_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn after_attribute_delete(
    dsl: &DSL<'_, T>,
    _old_attribute: &Attribute,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_ATTRIBUTE_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_potion_insert(
    dsl: &DSL<'_, T>,
    mut create_potion_request: CreatePotion,
) -> Result<CreatePotion, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_POTION_INSERT".to_string(),
    })?;

    create_potion_request.value = format!("{}_POTION", create_potion_request.value);

    Ok(create_potion_request)
}

#[spacetimedsl::hook]
fn after_potion_insert(dsl: &DSL<'_, T>, new_potion: &Potion) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_POTION_INSERT".to_string(),
    })?;

    dsl.create_recipe(CreateRecipe {
        value: format!("FIRST_PERMANENT_{}_INCREASE", new_potion.get_value()),
        potion_id: new_potion.get_id(),
    })?;

    dsl.create_recipe(CreateRecipe {
        value: format!("SECOND_PERMANENT_{}_INCREASE", new_potion.get_value()),
        potion_id: new_potion.get_id(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_potion_update(
    dsl: &DSL<'_, T>,
    _old_potion: &Potion,
    mut new_potion: Potion,
) -> Result<Potion, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_POTION_UPDATE".to_string(),
    })?;

    new_potion.value = format!("{}_POTION", new_potion.value);

    Ok(new_potion)
}

#[spacetimedsl::hook]
fn after_potion_update(
    dsl: &DSL<'_, T>,
    _old_potion: &Potion,
    _new_potion: &Potion,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_POTION_UPDATE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_potion_delete(dsl: &DSL<'_, T>, _old_potion: &Potion) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_POTION_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn after_potion_delete(dsl: &DSL<'_, T>, _old_potion: &Potion) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_POTION_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_recipe_insert(
    dsl: &DSL<'_, T>,
    mut create_recipe_request: CreateRecipe,
) -> Result<CreateRecipe, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_RECIPE_INSERT".to_string(),
    })?;

    create_recipe_request.value = format!("{}_RECIPE", create_recipe_request.value);

    Ok(create_recipe_request)
}

#[spacetimedsl::hook]
fn after_recipe_insert(dsl: &DSL<'_, T>, _new_recipe: &Recipe) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_RECIPE_INSERT".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_recipe_update(
    dsl: &DSL<'_, T>,
    _old_recipe: &Recipe,
    mut new_recipe: Recipe,
) -> Result<Recipe, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_RECIPE_UPDATE".to_string(),
    })?;

    new_recipe.value = format!("{}_RECIPE", new_recipe.value);

    Ok(new_recipe)
}

#[spacetimedsl::hook]
fn after_recipe_update(
    dsl: &DSL<'_, T>,
    _old_recipe: &Recipe,
    _new_recipe: &Recipe,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_RECIPE_UPDATE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_recipe_delete(dsl: &DSL<'_, T>, _old_recipe: &Recipe) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_RECIPE_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn after_recipe_delete(dsl: &DSL<'_, T>, _old_recipe: &Recipe) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_RECIPE_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_multi_column_index_with_hook_test_insert(
    dsl: &DSL<'_, T>,
    create_multi_column_index_with_hook_test_request: CreateMultiColumnIndexWithHookTest,
) -> Result<CreateMultiColumnIndexWithHookTest, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT".to_string(),
    })?;

    Ok(create_multi_column_index_with_hook_test_request)
}

#[spacetimedsl::hook]
fn after_multi_column_index_with_hook_test_insert(
    dsl: &DSL<'_, T>,
    _new_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_multi_column_index_with_hook_test_update(
    dsl: &DSL<'_, T>,
    _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
    new_multi_column_index_with_hook_test: MultiColumnIndexWithHookTest,
) -> Result<MultiColumnIndexWithHookTest, SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE".to_string(),
    })?;

    Ok(new_multi_column_index_with_hook_test)
}

#[spacetimedsl::hook]
fn after_multi_column_index_with_hook_test_update(
    dsl: &DSL<'_, T>,
    _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
    _new_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn before_multi_column_index_with_hook_test_delete(
    dsl: &DSL<'_, T>,
    _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE".to_string(),
    })?;

    Ok(())
}

#[spacetimedsl::hook]
fn after_multi_column_index_with_hook_test_delete(
    dsl: &DSL<'_, T>,
    _old_multi_column_index_with_hook_test: &MultiColumnIndexWithHookTest,
) -> Result<(), SpacetimeDSLError> {
    dsl.create_hook_call(CreateHookCall {
        value: "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE".to_string(),
    })?;

    Ok(())
}

// FIXME: Procedures can currently not be called through the SpacetimeDB CLI and not through reducers, only through clients. Will need to rely on successful compilation for now.
#[spacetimedb::procedure]
pub fn my_procedure(ctx: &mut ProcedureContext) {
    let result = ctx.try_with_tx(|ctx| {
        let dsl = dsl(ctx);

        run_tests(&dsl)
    });

    match result {
        Ok(_) => {}
        Err(msg) => panic!("{msg}"),
    }
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut strength = dsl.create_attribute(CreateAttribute {
        value: "STRENGTH".to_string(),
    })?;

    if strength.get_value().ne("STRENGTH_ATTRIBUTE") {
        return Err("Attribute value should be 'STRENGTH_ATTRIBUTE'".to_string());
    }

    let permanent_strength_attribute_increase_potion =
        dsl.get_potion_by_value("PERMANENT_STRENGTH_ATTRIBUTE_INCREASE_POTION")?;

    strength.set_value("POWER".to_string());

    let power = dsl.update_attribute_by_id(strength)?;

    if power.get_value().ne("POWER_ATTRIBUTE") {
        return Err(format!(
            "Attribute value should be 'POWER_ATTRIBUTE'. Got: {}",
            power.get_value()
        ));
    }

    let permanent_power_attribute_increase_potion =
        dsl.get_potion_by_id(permanent_strength_attribute_increase_potion.get_id())?;

    if permanent_power_attribute_increase_potion
        .get_value()
        .ne("PERMANENT_POWER_ATTRIBUTE_INCREASE_POTION")
    {
        return Err(
            "Potion value should be 'PERMANENT_POWER_ATTRIBUTE_INCREASE_POTION'".to_string(),
        );
    }

    dsl.delete_attribute_by_id(&power)?;

    if dsl.count_of_all_potions().ne(&0) {
        return Err("There should be 0 potions because the attribute was deleted and the potion should be deleted in the after delete hook of the attribute table.".to_string());
    }

    let mut multi_column_index_with_hook_test =
        dsl.create_multi_column_index_with_hook_test(CreateMultiColumnIndexWithHookTest {
            value_1: false,
            value_2: false,
        })?;

    multi_column_index_with_hook_test.set_value_1(true);
    multi_column_index_with_hook_test.set_value_2(true);

    multi_column_index_with_hook_test =
        dsl.update_multi_column_index_with_hook_test_by_id(multi_column_index_with_hook_test)?;

    dsl.delete_multi_column_index_with_hook_test_by_id(&multi_column_index_with_hook_test)?;

    let hook_calls: Vec<_> = dsl
        .get_all_hook_calls()
        .map(|hc| hc.get_value().to_string())
        .collect();

    let expected_hook_call_values = vec![
        "BEFORE_ATTRIBUTE_INSERT",
        "AFTER_ATTRIBUTE_INSERT",
        "BEFORE_POTION_INSERT",
        "AFTER_POTION_INSERT",
        "BEFORE_RECIPE_INSERT",
        "AFTER_RECIPE_INSERT",
        "BEFORE_RECIPE_INSERT",
        "AFTER_RECIPE_INSERT",
        "BEFORE_ATTRIBUTE_UPDATE",
        "AFTER_ATTRIBUTE_UPDATE",
        "BEFORE_POTION_UPDATE",
        "AFTER_POTION_UPDATE",
        "BEFORE_ATTRIBUTE_DELETE",
        "AFTER_ATTRIBUTE_DELETE",
        "BEFORE_POTION_DELETE",
        "AFTER_POTION_DELETE",
        "BEFORE_RECIPE_DELETE",
        "AFTER_RECIPE_DELETE",
        "BEFORE_RECIPE_DELETE",
        "AFTER_RECIPE_DELETE",
        "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT",
        "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_INSERT",
        "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE",
        "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_UPDATE",
        "BEFORE_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE",
        "AFTER_MULTI_COLUMN_INDEX_WITH_HOOK_TEST_DELETE",
    ];

    if hook_calls.ne(&expected_hook_call_values) {
        return Err(format!(
            "The hook calls do not match the expected ones!\n\nExpected:\n{:?}\n\nActual:\n{:?}",
            expected_hook_call_values, hook_calls
        ));
    }

    Ok(())
}
