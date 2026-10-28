//! References a `String` primary key from a foreign key of every shape: a column with a
//! non-unique index, a column with a unique index, a primary key, and a singleton's column.
//!
//! A lookup by a `String` key takes a `&str` rather than the key's wrapper, and a `String`
//! cannot be copied out of a borrowed row, unlike the `u64` and `Uuid` keys every other group
//! references. Only a module which is actually built proves that each generated call passes
//! such a key along.

use crate::spacetimedsl::prelude::*;

/// A country, keyed by its code. Its before-update hook makes `update_country_by_code` look
/// the stored row up by that key.
#[spacetimedsl::dsl(
    plural_name = countries,
    method(update = true, delete = true),
    hook(before(update))
)]
#[spacetimedb::table(accessor = country)]
pub struct Country {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::string_reference_test, table = city)]
    #[referenced_by(path = crate::string_reference_test, table = embassy)]
    #[referenced_by(path = crate::string_reference_test, table = country_profile)]
    #[referenced_by(path = crate::string_reference_test, table = home_country)]
    code: String,

    #[index(btree)]
    pub continent: String,
}

#[spacetimedsl::hook]
fn before_country_update(
    _dsl: &DSL<'_, T>,
    _old: &Country,
    new: Country,
) -> Result<Country, SpacetimeDSLError> {
    Ok(new)
}

#[spacetimedsl::dsl(plural_name = cities, method(update = true, delete = true))]
#[spacetimedb::table(accessor = city)]
pub struct City {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CountryCode)]
    #[foreign_key(path = crate::string_reference_test, table = country, column = code, on_delete = Delete)]
    pub country_code: String,
}

/// A country has at most one embassy, which keeps the country from being deleted.
#[spacetimedsl::dsl(plural_name = embassies, method(update = true, delete = true))]
#[spacetimedb::table(accessor = embassy)]
pub struct Embassy {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[use_wrapper(CountryCode)]
    #[foreign_key(path = crate::string_reference_test, table = country, column = code, on_delete = Error)]
    pub country_code: String,
}

/// Keyed by the code of the country it describes.
#[spacetimedsl::dsl(plural_name = country_profiles, method(update = true, delete = true))]
#[spacetimedb::table(accessor = country_profile)]
pub struct CountryProfile {
    #[primary_key]
    #[use_wrapper(CountryCode)]
    #[foreign_key(path = crate::string_reference_test, table = country, column = code, on_delete = Delete)]
    country_code: String,

    pub motto: String,
}

#[spacetimedsl::dsl(singleton(with_default), method(update = true, delete = true))]
#[spacetimedb::table(accessor = home_country)]
pub struct HomeCountry {
    #[use_wrapper(CountryCode)]
    #[foreign_key(path = crate::string_reference_test, table = country, column = code, on_delete = Delete)]
    pub country_code: String,
}

impl DefaultSingleton for HomeCountry {
    fn get_default(
        _dsl: &ReadOnlyDSL<'_, impl ReadContext>,
    ) -> Result<HomeCountry, SpacetimeDSLError> {
        Ok(HomeCountry {
            id: 0,
            // The default is never written, so it references no row.
            country_code: String::new(),
        })
    }
}

#[spacetimedsl::dsl(
    plural_name = provinces,
    method(update = true, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = province)]
pub struct Province {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::string_reference_test, table = district)]
    code: String,

    deleted: bool,
}

/// Both referenced and referencing, so a cascade reaching a district goes on to its villages.
#[spacetimedsl::dsl(
    plural_name = districts,
    method(update = true, delete = true, soft_delete = true)
)]
#[spacetimedb::table(accessor = district)]
pub struct District {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::string_reference_test, table = village)]
    code: String,

    #[index(btree)]
    #[use_wrapper(ProvinceCode)]
    #[foreign_key(path = crate::string_reference_test, table = province, column = code, on_delete = Delete, on_soft_delete = SoftDelete)]
    pub province_code: String,

    deleted: bool,
}

/// Its after-delete hook reads the village a cascade has just deleted.
#[spacetimedsl::dsl(
    plural_name = villages,
    method(update = true, delete = true),
    hook(after(delete))
)]
#[spacetimedb::table(accessor = village)]
pub struct Village {
    #[primary_key]
    #[create_wrapper]
    name: String,

    #[index(btree)]
    #[use_wrapper(DistrictCode)]
    #[foreign_key(path = crate::string_reference_test, table = district, column = code, on_delete = Delete, on_soft_delete = Ignore)]
    pub district_code: String,
}

#[spacetimedsl::hook]
fn after_village_delete(_dsl: &DSL<'_, T>, _village: &Village) -> Result<(), SpacetimeDSLError> {
    Ok(())
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    a_reference_is_checked_on_create_and_update(dsl)?;
    a_singleton_reference_is_checked_on_upsert(dsl)?;
    the_wrappers_follow_a_reference_both_ways(dsl)?;
    deleting_a_country_runs_the_strategies_of_its_references(dsl)?;
    deleting_many_countries_deletes_their_cities(dsl)?;
    a_country_is_updated_through_its_hook(dsl)?;
    deleting_a_province_deletes_its_districts_and_their_villages(dsl)?;
    soft_deleting_a_province_soft_deletes_its_districts(dsl)?;

    Ok(())
}

fn create_country<T: WriteContext>(
    dsl: &DSL<'_, T>,
    code: &str,
    continent: &str,
) -> Result<Country, SpacetimeDSLError> {
    dsl.create_country(CreateCountry {
        code: code.to_string(),
        continent: continent.to_string(),
    })
}

fn country_code(code: &str) -> CountryCode {
    CountryCode::new(code.to_string())
}

/// The "which row" part of a reference integrity violation on create or update, or `None`
/// for any other result.
fn violating_row<R>(result: Result<R, SpacetimeDSLError>) -> Option<String> {
    match result {
        Err(SpacetimeDSLError::ReferenceIntegrityViolation(
            ReferenceIntegrityViolationError::OnCreateOrUpdate {
                column_names_and_row_values,
                ..
            },
        )) => Some(column_names_and_row_values.to_string()),
        _ => None,
    }
}

fn a_reference_is_checked_on_create_and_update<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    create_country(dsl, "NL", "Europe")?;
    create_country(dsl, "BE", "Europe")?;

    let mut city = dsl
        .create_city(CreateCity {
            country_code: country_code("NL"),
        })
        .map_err(|error| {
            format!("Creating a city in an existing country should succeed: {error}")
        })?;

    let expected = "{ country_code : XX }";

    let violation = violating_row(dsl.create_city(CreateCity {
        country_code: country_code("XX"),
    }));
    if violation.as_deref() != Some(expected) {
        return Err(format!(
            "Creating a city in a missing country should fail naming `{expected}`, found {violation:?}!"
        ));
    }

    city.set_country_code(country_code("XX"));
    let violation = violating_row(dsl.update_city_by_id(city.clone()));
    if violation.as_deref() != Some(expected) {
        return Err(format!(
            "Moving a city into a missing country should fail naming `{expected}`, found {violation:?}!"
        ));
    }

    city.set_country_code(country_code("BE"));
    let city = dsl.update_city_by_id(city).map_err(|error| {
        format!("Moving a city into an existing country should succeed: {error}")
    })?;
    if city.get_country_code() != country_code("BE") {
        return Err(format!(
            "Moving a city into BE should write BE, found {}!",
            city.get_country_code()
        ));
    }

    Ok(())
}

fn a_singleton_reference_is_checked_on_upsert<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    create_country(dsl, "DK", "Europe")?;

    let mut home_country = dsl.get_home_country()?;
    home_country.set_country_code(country_code("XX"));

    let expected = "{ country_code : XX }";
    let violation = violating_row(dsl.upsert_home_country(home_country.clone()));
    if violation.as_deref() != Some(expected) {
        return Err(format!(
            "Inserting a home country which is missing should fail naming `{expected}`, found {violation:?}!"
        ));
    }

    home_country.set_country_code(country_code("DK"));
    let mut home_country = dsl
        .upsert_home_country(home_country)
        .map_err(|error| format!("Inserting an existing home country should succeed: {error}"))?;

    home_country.set_country_code(country_code("XX"));
    let violation = violating_row(dsl.upsert_home_country(home_country));
    if violation.as_deref() != Some(expected) {
        return Err(format!(
            "Updating the home country to a missing one should fail naming `{expected}`, found {violation:?}!"
        ));
    }

    Ok(())
}

fn the_wrappers_follow_a_reference_both_ways<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let country = create_country(dsl, "SE", "Europe")?;
    let city = dsl.create_city(CreateCity {
        country_code: country.get_code(),
    })?;
    let embassy = dsl.create_embassy(CreateEmbassy {
        country_code: country.get_code(),
    })?;
    dsl.create_country_profile(CreateCountryProfile {
        country_code: country.get_code(),
        motto: "For Sweden, with the times".to_string(),
    })?;

    let cities = country.get_code().get_cities(dsl);
    if cities.len() != 1 || cities[0].get_id() != city.get_id() {
        return Err(format!(
            "CountryCode::get_cities should find the one city of SE, found {cities:?}!"
        ));
    }

    if country.get_code().get_embassy(dsl)?.get_id() != embassy.get_id() {
        return Err("CountryCode::get_embassy should find the embassy of SE!".to_string());
    }

    if country.get_code().get_country_profile(dsl)?.get_motto() != "For Sweden, with the times" {
        return Err("CountryCode::get_country_profile should find the profile of SE!".to_string());
    }

    if city.get_id().get_country(dsl)?.get_code() != country.get_code() {
        return Err("CityId::get_country should find the country of the city!".to_string());
    }

    if embassy.get_id().get_country(dsl)?.get_code() != country.get_code() {
        return Err("EmbassyId::get_country should find the country of the embassy!".to_string());
    }

    Ok(())
}

/// `Error` keeps the country while an embassy references it, and `Delete` removes every other
/// row referencing it once it is gone.
fn deleting_a_country_runs_the_strategies_of_its_references<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let country = create_country(dsl, "FI", "Europe")?;
    let city = dsl.create_city(CreateCity {
        country_code: country.get_code(),
    })?;
    let embassy = dsl.create_embassy(CreateEmbassy {
        country_code: country.get_code(),
    })?;
    dsl.create_country_profile(CreateCountryProfile {
        country_code: country.get_code(),
        motto: "Sisu".to_string(),
    })?;
    let mut home_country = dsl.get_home_country()?;
    home_country.set_country_code(country.get_code());
    dsl.upsert_home_country(home_country)?;

    if dsl.delete_country_by_code("FI").is_ok() {
        return Err("Deleting a country with an embassy should fail!".to_string());
    }

    dsl.delete_embassy_by_id(embassy.get_id())?;
    dsl.delete_country_by_code("FI").map_err(|error| {
        format!("Deleting a country without an embassy should succeed: {error}")
    })?;

    if dsl.get_city_by_id(city.get_id()).is_ok() {
        return Err("Deleting a country should delete its city!".to_string());
    }

    if dsl.get_country_profile_by_country_code("FI").is_ok() {
        return Err("Deleting a country should delete its profile!".to_string());
    }

    if dsl.get_home_country()?.get_country_code() == country.get_code() {
        return Err("Deleting the home country should delete the row naming it!".to_string());
    }

    Ok(())
}

fn deleting_many_countries_deletes_their_cities<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let continent = "Atlantis";
    let first_country = create_country(dsl, "A1", continent)?;
    let second_country = create_country(dsl, "A2", continent)?;
    let first_city = dsl.create_city(CreateCity {
        country_code: first_country.get_code(),
    })?;
    let second_city = dsl.create_city(CreateCity {
        country_code: second_country.get_code(),
    })?;

    dsl.delete_countries_by_continent(continent)
        .map_err(|error| {
            format!("Deleting the countries of a continent should succeed: {error}")
        })?;

    if dsl.get_city_by_id(first_city.get_id()).is_ok()
        || dsl.get_city_by_id(second_city.get_id()).is_ok()
    {
        return Err(
            "Deleting the countries of a continent should delete their cities!".to_string(),
        );
    }

    Ok(())
}

fn a_country_is_updated_through_its_hook<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut country = create_country(dsl, "IS", "Europe")?;
    country.set_continent("North America".to_string());

    let country = dsl
        .update_country_by_code(country)
        .map_err(|error| format!("Updating a country should succeed: {error}"))?;

    if country.get_continent() != "North America" {
        return Err(format!(
            "Updating a country should write its new continent, found {}!",
            country.get_continent()
        ));
    }

    Ok(())
}

fn create_district<T: WriteContext>(
    dsl: &DSL<'_, T>,
    code: &str,
    province: &Province,
) -> Result<District, SpacetimeDSLError> {
    dsl.create_district(CreateDistrict {
        code: code.to_string(),
        province_code: province.get_code(),
    })
}

fn create_village<T: WriteContext>(
    dsl: &DSL<'_, T>,
    name: &str,
    district: &District,
) -> Result<Village, SpacetimeDSLError> {
    dsl.create_village(CreateVillage {
        name: name.to_string(),
        district_code: district.get_code(),
    })
}

fn deleting_a_province_deletes_its_districts_and_their_villages<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let province = dsl.create_province(CreateProvince {
        code: "P1".to_string(),
    })?;
    let first_district = create_district(dsl, "P1-D1", &province)?;
    let second_district = create_district(dsl, "P1-D2", &province)?;
    create_village(dsl, "P1-D1-V1", &first_district)?;
    create_village(dsl, "P1-D2-V1", &second_district)?;

    dsl.delete_province_by_code("P1")
        .map_err(|error| format!("Deleting a province should succeed: {error}"))?;

    if dsl.get_district_by_code("P1-D1").is_ok() || dsl.get_district_by_code("P1-D2").is_ok() {
        return Err("Deleting a province should delete its districts!".to_string());
    }

    if dsl.get_village_by_name("P1-D1-V1").is_ok() || dsl.get_village_by_name("P1-D2-V1").is_ok() {
        return Err("Deleting a province should delete the villages of its districts!".to_string());
    }

    Ok(())
}

fn soft_deleting_a_province_soft_deletes_its_districts<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let province = dsl.create_province(CreateProvince {
        code: "P2".to_string(),
    })?;
    let district = create_district(dsl, "P2-D1", &province)?;
    create_village(dsl, "P2-D1-V1", &district)?;

    dsl.soft_delete_province_by_code("P2")
        .map_err(|error| format!("Soft-deleting a province should succeed: {error}"))?;

    if !*dsl.get_district_by_code("P2-D1")?.get_deleted() {
        return Err("Soft-deleting a province should soft-delete its district!".to_string());
    }

    if dsl.get_village_by_name("P2-D1-V1").is_err() {
        return Err(
            "Soft-deleting a province should keep the villages of its districts!".to_string(),
        );
    }

    Ok(())
}
