//! A table whose rows are never removed, `method(delete = false)` without
//! `method(soft_delete = true)`, can be referenced. Its foreign keys declare no strategy,
//! because there is no removal to react to, and create and update still check that they
//! reference a row.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = catalog_categories, method(update = false, delete = false))]
#[spacetimedb::table(accessor = catalog_category)]
pub struct CatalogCategory {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(CatalogCategoryId)]
    #[referenced_by(path = crate::referenced_table_without_delete_methods_test, table = catalog_product)]
    #[referenced_by(path = crate::referenced_table_without_delete_methods_test, table = catalog_bundle)]
    id: u64,
}

/// Two foreign keys to one table, whose pairing imports have to stay deduplicated.
#[spacetimedsl::dsl(plural_name = catalog_products, method(update = true, delete = true))]
#[spacetimedb::table(accessor = catalog_product)]
pub struct CatalogProduct {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CatalogCategoryId)]
    #[foreign_key(
        path = crate::referenced_table_without_delete_methods_test,
        table = catalog_category,
        column = id
    )]
    pub category_id: u64,

    #[index(btree)]
    #[use_wrapper(CatalogCategoryId)]
    #[foreign_key(
        path = crate::referenced_table_without_delete_methods_test,
        table = catalog_category,
        column = id
    )]
    pub secondary_category_id: u64,
}

/// A second table of the same module referencing the same table, whose pairing imports must
/// not clash with those of `CatalogProduct`.
#[spacetimedsl::dsl(plural_name = catalog_bundles, method(update = false, delete = true))]
#[spacetimedb::table(accessor = catalog_bundle)]
pub struct CatalogBundle {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CatalogCategoryId)]
    #[foreign_key(
        path = crate::referenced_table_without_delete_methods_test,
        table = catalog_category,
        column = id
    )]
    category_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    create_checks_the_reference(dsl)?;
    update_checks_the_reference(dsl)?;

    Ok(())
}

/// Creating rows which reference an existing category works, and creating one which
/// references no category is rejected.
fn create_checks_the_reference<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let category = dsl.create_catalog_category()?;

    dsl.create_catalog_product(CreateCatalogProduct {
        category_id: category.get_id(),
        secondary_category_id: category.get_id(),
    })
    .map_err(|error| {
        format!("A product referencing an existing category should be created! Got:\n{error}")
    })?;
    dsl.create_catalog_bundle(CreateCatalogBundle {
        category_id: category.get_id(),
    })
    .map_err(|error| {
        format!("A bundle referencing an existing category should be created! Got:\n{error}")
    })?;

    match dsl.create_catalog_product(CreateCatalogProduct {
        category_id: CatalogCategoryId::new(u64::MAX),
        secondary_category_id: category.get_id(),
    }) {
        Err(SpacetimeDSLError::ReferenceIntegrityViolation(_)) => Ok(()),
        other => Err(format!(
            "A product whose category_id references no category should be rejected! Got:\n{other:?}"
        )),
    }
}

/// Updating a product so that a foreign key references no category is rejected.
fn update_checks_the_reference<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let category = dsl.create_catalog_category()?;
    let mut product = dsl.create_catalog_product(CreateCatalogProduct {
        category_id: category.get_id(),
        secondary_category_id: category.get_id(),
    })?;
    product.set_secondary_category_id(CatalogCategoryId::new(u64::MAX));

    match dsl.update_catalog_product_by_id(product) {
        Err(SpacetimeDSLError::ReferenceIntegrityViolation(_)) => Ok(()),
        other => Err(format!(
            "Updating a product so that secondary_category_id references no category should be rejected! Got:\n{other:?}"
        )),
    }
}
