//! When step definitions
//!
//! Steps that execute actions (law evaluations).

use cucumber::when;
use regelrecht_engine::Value;

use crate::world::RegelrechtWorld;

// =============================================================================
// Bijstand steps
// =============================================================================

#[when(regex = r"^the bijstandsaanvraag is executed for participatiewet article (\d+)$")]
fn execute_bijstandsaanvraag(world: &mut RegelrechtWorld, _article: String) {
    // The bijstand calculation uses article 43 which produces uitkering_bedrag
    // We execute the law for the "uitkering_bedrag" output
    world.execute_law("participatiewet", "uitkering_bedrag");
}

// =============================================================================
// Erfgrensbeplanting steps
// =============================================================================

#[when(regex = r"^the erfgrensbeplanting is requested for (\S+) article (\d+)$")]
fn execute_erfgrensbeplanting(world: &mut RegelrechtWorld, law_id: String, _article: String) {
    // Execute for minimale_afstand_cm output
    world.execute_law(&law_id, "minimale_afstand_cm");
}

// =============================================================================
// Zorgtoeslag steps
// =============================================================================

#[when(regex = r"^I request the standard premium for year (\d+)$")]
fn request_standard_premium(world: &mut RegelrechtWorld, year: String) {
    // Set the calculation date to the requested year
    world.calculation_date = format!("{}-01-01", year);

    // The standard premium is defined in regeling_standaardpremie
    world.execute_law("regeling_standaardpremie", "standaardpremie");
}

#[when("the healthcare allowance law is executed")]
fn execute_healthcare_allowance(world: &mut RegelrechtWorld) {
    use std::collections::HashMap;

    // Determine BSN from parameters or external data
    let bsn = world
        .parameters
        .get("bsn")
        .or_else(|| world.parameters.get("BSN"))
        .cloned()
        .unwrap_or(Value::String("unknown".to_string()));

    // Build a derived data record keyed by BSN.
    // These fields are registered in the DataSourceRegistry so the engine
    // resolves them during input resolution for Article 2 (top-level).
    let mut derived_record: HashMap<String, Value> = HashMap::new();
    derived_record.insert("bsn".to_string(), bsn);

    // Calculate age from geboortedatum if present
    if let Some(Value::String(dob)) = world
        .external_data
        .rvig_personal
        .values()
        .next()
        .and_then(|d| d.get("geboortedatum"))
        .or_else(|| world.parameters.get("geboortedatum"))
        .cloned()
    {
        if let (Some(dob_year), Some(calc_year)) = (
            dob.split('-').next().and_then(|y| y.parse::<i64>().ok()),
            world
                .calculation_date
                .split('-')
                .next()
                .and_then(|y| y.parse::<i64>().ok()),
        ) {
            let age = calc_year - dob_year;
            derived_record.insert("leeftijd".to_string(), Value::Int(age));
        }
    }

    // Set has_partner based on partnerschap_type
    let has_partner = if let Some(Value::String(pt)) = world
        .external_data
        .rvig_relationship
        .values()
        .next()
        .and_then(|d| d.get("partnerschap_type"))
        .or_else(|| world.parameters.get("partnerschap_type"))
        .cloned()
    {
        let hp = pt != "GEEN";
        derived_record.insert("heeft_toeslagpartner".to_string(), Value::Bool(hp));
        Some(hp)
    } else {
        None
    };

    // Set insurance status
    // Note: law input name is "is_verzekerde" (with trailing 'e')
    if let Some(Value::String(status)) = world
        .external_data
        .rvz_insurance
        .values()
        .next()
        .and_then(|d| d.get("polis_status"))
        .or_else(|| world.parameters.get("polis_status"))
        .cloned()
    {
        let is_insured = status == "ACTIEF";
        derived_record.insert("is_verzekerde".to_string(), Value::Bool(is_insured));
    }

    // Calculate toetsingsinkomen from box1, box2, box3 external data
    let mut toetsingsinkomen: i64 = 0;

    // Box 1 components
    let box1_data = world.external_data.bd_box1.values().next();
    for key in [
        "loon_uit_dienstbetrekking",
        "uitkeringen_en_pensioenen",
        "winst_uit_onderneming",
        "resultaat_overige_werkzaamheden",
        "eigen_woning",
    ] {
        if let Some(v) = box1_data
            .and_then(|d| d.get(key))
            .or_else(|| world.parameters.get(key))
        {
            if let Some(n) = v.as_int() {
                toetsingsinkomen += n;
            }
        }
    }

    // Box 2 components
    let box2_data = world.external_data.bd_box2.values().next();
    for key in ["reguliere_voordelen", "vervreemdingsvoordelen"] {
        if let Some(v) = box2_data
            .and_then(|d| d.get(key))
            .or_else(|| world.parameters.get(key))
        {
            if let Some(n) = v.as_int() {
                toetsingsinkomen += n;
            }
        }
    }

    // Box 3 is capital (used for vermogen)
    let box3_data = world.external_data.bd_box3.values().next();
    let mut box3_total: i64 = 0;
    for key in ["spaargeld", "beleggingen", "onroerend_goed"] {
        if let Some(v) = box3_data
            .and_then(|d| d.get(key))
            .or_else(|| world.parameters.get(key))
        {
            if let Some(n) = v.as_int() {
                box3_total += n;
            }
        }
    }
    if let Some(v) = box3_data
        .and_then(|d| d.get("schulden"))
        .or_else(|| world.parameters.get("schulden"))
    {
        if let Some(n) = v.as_int() {
            box3_total -= n;
        }
    }

    derived_record.insert("toetsingsinkomen".to_string(), Value::Int(toetsingsinkomen));

    // Register Article 2's external inputs in the DataSourceRegistry.
    // The engine will resolve leeftijd, is_verzekerde, heeft_toeslagpartner,
    // and toetsingsinkomen from this registry instead of cross-law resolution.
    world
        .service
        .register_dict_source("derived_zorgtoeslag_inputs", "bsn", vec![derived_record])
        .expect("Failed to register derived data source");

    // Article 3 (vermogen_onder_grens) is resolved internally by ArticleEngine,
    // which doesn't consult the DataSourceRegistry. Its inputs (vermogen,
    // heeft_toeslagpartner) must be in parameters to propagate via combined_params.
    world
        .parameters
        .insert("vermogen".to_string(), Value::Int(box3_total));
    if let Some(hp) = has_partner {
        world
            .parameters
            .insert("heeft_toeslagpartner".to_string(), Value::Bool(hp));
    }

    // Execute the healthcare allowance calculation.
    // BSN stays in parameters (required article parameter).
    // Article 2 inputs (leeftijd, is_verzekerde, heeft_toeslagpartner,
    // toetsingsinkomen) are resolved from the registry.
    // standaardpremie is resolved via cross-law (regeling_standaardpremie).
    // vermogen_onder_grens is an internal reference resolved by ArticleEngine.
    world.execute_law("zorgtoeslagwet", "hoogte_zorgtoeslag");
}
