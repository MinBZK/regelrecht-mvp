//! The game in the browser: [`Spel`] behind wasm-bindgen, one [`Poc`] per
//! tab.
//!
//! The page gives the bundle it was built with ([`crate::bundel`]) and calls
//! one method per step. Every method answers in JSON text, as the HTTP API
//! did before the app moved into the browser, and a refusal is a thrown
//! `Error` with the cell's message. Structured arguments (`velden`,
//! `bevindingen`, a saved game) come in as JSON text too, so a value reaches
//! the cell exactly as the page wrote it, numbers and all.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;

use crate::spel::{Antwoord, Spel};

#[wasm_bindgen]
pub struct Poc {
    spel: Spel,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn console_error(message: &str);
}

/// A panic aborts the WebAssembly instance, and the page only sees that it
/// stopped (api.js says so). The message goes to the console, so whoever
/// looks there sees why.
fn meld_panics() {
    static EENMAAL: std::sync::Once = std::sync::Once::new();
    EENMAAL.call_once(|| std::panic::set_hook(Box::new(|info| console_error(&info.to_string()))));
}

fn fout(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

fn antwoord(a: Antwoord) -> Result<String, JsError> {
    let value = a.map_err(fout)?;
    serde_json::to_string(&value).map_err(fout)
}

fn dag(datum: &str) -> Result<NaiveDate, JsError> {
    datum
        .parse()
        .map_err(|e| fout(format!("'{datum}' is no date (YYYY-MM-DD): {e}")))
}

/// The JSON object `json` given as `naam`.
fn kaart(naam: &str, json: &str) -> Result<Map<String, Value>, JsError> {
    serde_json::from_str(json).map_err(|e| fout(format!("{naam}: {e}")))
}

#[wasm_bindgen]
impl Poc {
    /// The game on `bundel`, the JSON object `{ "<path>": "<text>" }` the
    /// page was built with. A bundle that misses a file a case needs, or a
    /// case the law does not support, fails here, before anything is shown.
    #[wasm_bindgen(constructor)]
    pub fn new(bundel: &str) -> Result<Poc, JsError> {
        meld_panics();
        let files: BTreeMap<String, String> =
            serde_json::from_str(bundel).map_err(|e| fout(format!("the bundle: {e}")))?;
        let casussen = crate::bundel::casussen(files).map_err(fout)?;
        Ok(Poc {
            spel: Spel::new(casussen),
        })
    }

    pub fn casussen(&self) -> Result<String, JsError> {
        antwoord(self.spel.casussen())
    }

    pub fn casus(&mut self) -> Result<String, JsError> {
        antwoord(self.spel.casus())
    }

    pub fn kies(&mut self, casus: &str) -> Result<String, JsError> {
        antwoord(self.spel.kiezen(casus))
    }

    pub fn vormen(&mut self, datum: &str) -> Result<String, JsError> {
        antwoord(self.spel.vormen(dag(datum)?))
    }

    pub fn kroniek(&mut self) -> Result<String, JsError> {
        antwoord(self.spel.kroniek())
    }

    pub fn aanvragen(&mut self, velden: &str, datum: &str) -> Result<String, JsError> {
        antwoord(self.spel.aanvragen(&kaart("velden", velden)?, dag(datum)?))
    }

    pub fn vastleggen(
        &mut self,
        cel: &str,
        feit: &str,
        velden: &str,
        datum: &str,
    ) -> Result<String, JsError> {
        antwoord(
            self.spel
                .vastleggen(cel, feit, &kaart("velden", velden)?, dag(datum)?),
        )
    }

    pub fn besluiten(
        &mut self,
        op: &str,
        bevindingen: &str,
        datum: &str,
    ) -> Result<String, JsError> {
        antwoord(
            self.spel
                .besluiten(op, &kaart("bevindingen", bevindingen)?, dag(datum)?),
        )
    }

    #[wasm_bindgen(js_name = voorvertoningBesluit)]
    pub fn voorvertoning_besluit(
        &mut self,
        op: &str,
        bevindingen: &str,
        datum: &str,
    ) -> Result<String, JsError> {
        antwoord(self.spel.voorvertoning_besluit(
            op,
            &kaart("bevindingen", bevindingen)?,
            dag(datum)?,
        ))
    }

    pub fn waarom(&mut self, id: &str) -> Result<String, JsError> {
        antwoord(self.spel.waarom(id))
    }

    #[wasm_bindgen(js_name = hoeBeantwoord)]
    pub fn hoe_beantwoord(&mut self, id: &str, begrip: &str) -> Result<String, JsError> {
        antwoord(self.spel.hoe_beantwoord(id, begrip))
    }

    pub fn bezwaren(
        &mut self,
        besluit: &str,
        velden: &str,
        datum: &str,
    ) -> Result<String, JsError> {
        antwoord(
            self.spel
                .bezwaren(besluit, &kaart("velden", velden)?, dag(datum)?),
        )
    }

    pub fn heroverweging(&mut self, bezwaar: &str, datum: &str) -> Result<String, JsError> {
        antwoord(self.spel.heroverweging(bezwaar, dag(datum)?))
    }

    #[wasm_bindgen(js_name = beslissenOpBezwaar)]
    pub fn beslissen_op_bezwaar(
        &mut self,
        op: &str,
        bevindingen: &str,
        datum: &str,
    ) -> Result<String, JsError> {
        antwoord(self.spel.beslissen_op_bezwaar(
            op,
            &kaart("bevindingen", bevindingen)?,
            dag(datum)?,
        ))
    }

    #[wasm_bindgen(js_name = voorvertoningOpBezwaar)]
    pub fn voorvertoning_op_bezwaar(
        &mut self,
        op: &str,
        bevindingen: &str,
        datum: &str,
    ) -> Result<String, JsError> {
        antwoord(self.spel.voorvertoning_op_bezwaar(
            op,
            &kaart("bevindingen", bevindingen)?,
            dag(datum)?,
        ))
    }

    pub fn reducties(&mut self, datum: &str) -> Result<String, JsError> {
        antwoord(self.spel.reducties(dag(datum)?))
    }

    pub fn logboek(&mut self) -> Result<String, JsError> {
        antwoord(self.spel.logboek())
    }

    pub fn reset(&mut self) -> Result<String, JsError> {
        antwoord(self.spel.reset())
    }

    pub fn bewaren(&mut self) -> Result<String, JsError> {
        antwoord(self.spel.bewaren())
    }

    pub fn herstellen(&mut self, bewaard: &str) -> Result<String, JsError> {
        let bewaard: Value =
            serde_json::from_str(bewaard).map_err(|e| fout(format!("the saved game: {e}")))?;
        antwoord(self.spel.herstellen(bewaard))
    }
}
