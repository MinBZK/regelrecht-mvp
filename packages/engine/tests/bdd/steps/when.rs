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
    // Calculate age from geboortedatum if present
    if let Some(Value::String(dob)) = world.parameters.get("geboortedatum").cloned() {
        // Simple age calculation: extract year and compare to calculation date year
        if let (Some(dob_year), Some(calc_year)) = (
            dob.split('-').next().and_then(|y| y.parse::<i64>().ok()),
            world
                .calculation_date
                .split('-')
                .next()
                .and_then(|y| y.parse::<i64>().ok()),
        ) {
            let age = calc_year - dob_year;
            world
                .parameters
                .insert("leeftijd".to_string(), Value::Int(age));
        }
    }

    // Set has_partner based on partnerschap_type
    if let Some(Value::String(pt)) = world.parameters.get("partnerschap_type").cloned() {
        let has_partner = pt != "GEEN";
        world
            .parameters
            .insert("heeft_toeslagpartner".to_string(), Value::Bool(has_partner));
    }

    // Set insurance status
    if let Some(Value::String(status)) = world.parameters.get("polis_status").cloned() {
        let is_insured = status == "ACTIEF";
        world
            .parameters
            .insert("is_verzekerd".to_string(), Value::Bool(is_insured));
    }

    // Set residence in Netherlands based on land_verblijf
    if let Some(Value::String(land)) = world.parameters.get("land_verblijf").cloned() {
        let in_nl = land == "NEDERLAND";
        world
            .parameters
            .insert("woont_in_nederland".to_string(), Value::Bool(in_nl));
    }

    // Calculate toetsingsinkomen from box1, box2, box3 data
    let mut toetsingsinkomen: i64 = 0;

    // Box 1 components
    for key in [
        "loon_uit_dienstbetrekking",
        "uitkeringen_en_pensioenen",
        "winst_uit_onderneming",
        "resultaat_overige_werkzaamheden",
        "eigen_woning",
    ] {
        if let Some(v) = world.parameters.get(key) {
            if let Some(n) = v.as_int() {
                toetsingsinkomen += n;
            }
        }
    }

    // Box 2 components
    for key in ["reguliere_voordelen", "vervreemdingsvoordelen"] {
        if let Some(v) = world.parameters.get(key) {
            if let Some(n) = v.as_int() {
                toetsingsinkomen += n;
            }
        }
    }

    // Box 3 is capital, typically calculated differently but for simplicity
    // add spaargeld + beleggingen - schulden
    let mut box3_total: i64 = 0;
    for key in ["spaargeld", "beleggingen", "onroerend_goed"] {
        if let Some(v) = world.parameters.get(key) {
            if let Some(n) = v.as_int() {
                box3_total += n;
            }
        }
    }
    if let Some(v) = world.parameters.get("schulden") {
        if let Some(n) = v.as_int() {
            box3_total -= n;
        }
    }

    world
        .parameters
        .insert("toetsingsinkomen".to_string(), Value::Int(toetsingsinkomen));
    world
        .parameters
        .insert("vermogen".to_string(), Value::Int(box3_total));

    // Execute the healthcare allowance calculation
    // Note: The law ID is "zorgtoeslagwet", not "wet_op_de_zorgtoeslag"
    world.execute_law("zorgtoeslagwet", "hoogte_zorgtoeslag");
}
