//! The queue: one task at a time. A task is a law through time (`keten`) or
//! one Kamerdossier. What a visitor asks for goes before what the queue
//! fetches in the background; the same task is never in it twice. After a
//! law, the queue fetches its dossiers in the background, newest first, and
//! rebuilds the law after each one, so that the dossier's toelichting hangs
//! at its transition. A dossier that fails is remembered and not retried by
//! itself.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::Notify;

use crate::werker::{self, Regel};
use crate::AppState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Wat {
    Keten,
    Dossier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Wacht,
    Bezig,
    Klaar,
    Fout,
}

#[derive(Clone, Debug, Serialize)]
pub struct Taak {
    pub id: String,
    pub wat: Wat,
    pub bwb: String,
    pub dossier: Option<String>,
    /// The law the queue fetched this dossier for; it is rebuilt afterwards.
    pub voor: Option<String>,
    pub status: Status,
    pub uitkomst: Option<String>,
    pub fout: Option<String>,
    #[serde(skip)]
    pub log: Vec<String>,
}

impl Taak {
    fn sleutel(&self) -> (Wat, Option<&str>, Option<&str>) {
        let bwb = (self.wat == Wat::Keten).then_some(self.bwb.as_str());
        (self.wat, bwb, self.dossier.as_deref())
    }
    fn kort(&self) -> Value {
        json!({ "id": self.id, "wat": self.wat, "bwb": self.bwb, "dossier": self.dossier, "voor": self.voor, "status": self.status })
    }
}

#[derive(Default)]
struct Binnen {
    taken: HashMap<String, Taak>,
    /// The order in which tasks were added, to forget the oldest.
    volgorde: VecDeque<String>,
    wachtend: VecDeque<String>,
    huidig: Option<String>,
}

#[derive(Default)]
pub struct Rij {
    binnen: Mutex<Binnen>,
    wekker: Notify,
    teller: AtomicU64,
}

impl Rij {
    fn binnen(&self) -> std::sync::MutexGuard<'_, Binnen> {
        self.binnen.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Add a task, or return the one that is already waiting or running.
    pub fn voeg(
        &self,
        wat: Wat,
        bwb: &str,
        dossier: Option<&str>,
        voor: Option<&str>,
        achtergrond: bool,
    ) -> Value {
        let n = self.teller.fetch_add(1, Ordering::Relaxed) + 1;
        let taak = Taak {
            id: format!("{}-{n}", chrono::Utc::now().timestamp()),
            wat,
            bwb: bwb.into(),
            dossier: dossier.map(String::from),
            voor: voor.map(String::from),
            status: Status::Wacht,
            uitkomst: None,
            fout: None,
            log: Vec::new(),
        };
        let mut b = self.binnen();
        let bestaand = b
            .huidig
            .iter()
            .chain(b.wachtend.iter())
            .find(|id| b.taken[*id].sleutel() == taak.sleutel())
            .cloned();
        if let Some(id) = bestaand {
            // What a visitor asks for now goes before the background.
            if !achtergrond && b.taken[&id].voor.is_some() {
                if let Some(i) = b.wachtend.iter().position(|x| *x == id) {
                    b.wachtend.remove(i);
                    b.wachtend.push_front(id.clone());
                }
            }
            return Self::json(&b, &id);
        }
        let id = taak.id.clone();
        if achtergrond {
            b.wachtend.push_back(id.clone());
        } else {
            // Before the background, behind what other visitors asked for.
            let i = b
                .wachtend
                .iter()
                .position(|x| b.taken[x].voor.is_some())
                .unwrap_or(b.wachtend.len());
            b.wachtend.insert(i, id.clone());
        }
        b.taken.insert(id.clone(), taak);
        b.volgorde.push_back(id.clone());
        while b.volgorde.len() > 100 {
            let oud = b.volgorde.pop_front().unwrap_or_default();
            if b.taken
                .get(&oud)
                .is_some_and(|t| matches!(t.status, Status::Klaar | Status::Fout))
            {
                b.taken.remove(&oud);
            }
        }
        let antwoord = Self::json(&b, &id);
        drop(b);
        self.wekker.notify_one();
        antwoord
    }

    fn json(b: &Binnen, id: &str) -> Value {
        let t = &b.taken[id];
        let mut v = serde_json::to_value(t).unwrap_or_default();
        v["plaats"] = b
            .wachtend
            .iter()
            .position(|x| x == id)
            .map(|i| i + 1)
            .into();
        let log: Vec<&String> = t.log.iter().filter(|r| !r.trim().is_empty()).collect();
        v["log"] = json!(log[log.len().saturating_sub(40)..]);
        v
    }

    pub fn taak(&self, id: &str) -> Option<Value> {
        let b = self.binnen();
        b.taken.contains_key(id).then(|| Self::json(&b, id))
    }

    pub fn huidig(&self) -> Option<Value> {
        let b = self.binnen();
        b.huidig.as_deref().map(|id| Self::json(&b, id))
    }

    pub fn overzicht(&self) -> Value {
        let b = self.binnen();
        json!({
            "huidig": b.huidig.as_ref().map(|id| b.taken[id].kort()),
            "wachtend": b.wachtend.iter().map(|id| b.taken[id].kort()).collect::<Vec<_>>(),
        })
    }

    /// Whether a dossier is being fetched (`bezig`) or waits (`in de rij`).
    pub fn in_de_rij(&self, nr: &str) -> Option<&'static str> {
        let b = self.binnen();
        let is = |id: &String| b.taken[id].dossier.as_deref() == Some(nr);
        if b.huidig.as_ref().is_some_and(is) {
            Some("bezig")
        } else if b.wachtend.iter().any(is) {
            Some("in de rij")
        } else {
            None
        }
    }

    fn wijzig(&self, id: &str, f: impl FnOnce(&mut Taak)) {
        if let Some(t) = self.binnen().taken.get_mut(id) {
            f(t);
        }
    }

    fn get(&self, id: &str) -> Option<Taak> {
        self.binnen().taken.get(id).cloned()
    }

    /// The next task, waiting until there is one. It is `bezig` at once, so
    /// that the queue never looks empty between taking a task and running it.
    async fn volgende(&self) -> String {
        loop {
            {
                let mut b = self.binnen();
                if let Some(id) = b.wachtend.pop_front() {
                    if let Some(t) = b.taken.get_mut(&id) {
                        t.status = Status::Bezig;
                    }
                    b.huidig = Some(id.clone());
                    return id;
                }
                b.huidig = None;
            }
            self.wekker.notified().await;
        }
    }
}

/// The worker thread of the queue: one task after the other, for ever.
pub async fn werk(state: AppState) {
    loop {
        let id = state.rij.volgende().await;
        let Some(taak) = state.rij.get(&id) else {
            continue;
        };
        let t0 = std::time::Instant::now();
        state.logboek.schrijf("taak", json!({ "id": id, "wat": taak.wat, "bwb": taak.bwb, "dossier": taak.dossier, "voor": taak.voor, "status": "bezig" }));
        let uitkomst = draai(&state, &taak).await;
        state.rij.wijzig(&id, |t| match &uitkomst {
            Ok(u) => {
                t.status = Status::Klaar;
                t.uitkomst = Some(u.clone());
            }
            Err(e) => {
                t.status = Status::Fout;
                t.fout = Some(e.clone());
            }
        });
        if let (Err(e), Some(nr)) = (&uitkomst, &taak.dossier) {
            state.opslag.zet_mislukt(nr, e);
        }
        state.logboek.schrijf(
            "taak",
            json!({ "id": id, "wat": taak.wat, "bwb": taak.bwb, "dossier": taak.dossier, "voor": taak.voor,
                    "status": if uitkomst.is_ok() { "klaar" } else { "fout" },
                    "s": (t0.elapsed().as_secs_f64() * 10.0).round() / 10.0,
                    "uitkomst": uitkomst.as_ref().ok(), "fout": uitkomst.as_ref().err() }),
        );
    }
}

/// One task: the worker, then storing what it made.
async fn draai(state: &AppState, taak: &Taak) -> Result<String, String> {
    let id = taak.id.clone();
    let log = |r: &str| state.rij.wijzig(&id, |t| t.log.push(r.to_string()));
    let uit = state.opslag.tijdelijk().join(format!("{id}.json"));
    std::fs::create_dir_all(state.opslag.tijdelijk()).map_err(|e| e.to_string())?;
    let mappen = state.opslag.mappen();
    let regel = |r: Regel| match r {
        Regel::Log(r) => log(&r),
        Regel::Uit(v) => {
            if let Some(r) = werker::mislukte_vraag(&v) {
                log(&r);
            }
            state.logboek.schrijf("uit", v);
        }
    };
    match taak.wat {
        Wat::Keten => {
            let status = schrijf_status(state, &id)?;
            werker::draai(
                state,
                &[
                    "keten",
                    &taak.bwb,
                    "--uit",
                    werker::pad(&uit),
                    "--gegevens",
                    &mappen,
                    "--status",
                    werker::pad(&status),
                ],
                None,
                regel,
            )
            .await?;
            let naam = bewaar(state, &uit, None)?;
            let n = verdiep(state, &taak.bwb);
            log(&format!(
                "Klaar: {naam}{}",
                if n > 0 {
                    format!("; de rij haalt nog {n} Kamerdossier(s) op")
                } else {
                    String::new()
                }
            ));
            Ok(taak.bwb.clone())
        }
        Wat::Dossier => {
            let nr = taak.dossier.as_deref().unwrap_or_default();
            werker::draai(
                state,
                &["dossier", &taak.bwb, nr, "--uit", werker::pad(&uit)],
                None,
                regel,
            )
            .await?;
            let naam = bewaar(state, &uit, taak.voor.as_deref())?;
            state.opslag.wis_mislukt(nr);
            log(&format!("Klaar: {naam}"));
            if let Some(voor) = taak
                .voor
                .as_deref()
                .filter(|v| state.opslag.keten(v).is_some())
            {
                log(&format!(
                    "{voor} opnieuw opbouwen, met de toelichting van dossier {nr}"
                ));
                let status = schrijf_status(state, &id)?;
                werker::draai(
                    state,
                    &[
                        "opbouw",
                        voor,
                        "--uit",
                        werker::pad(&uit),
                        "--gegevens",
                        &mappen,
                        "--status",
                        werker::pad(&status),
                    ],
                    None,
                    regel,
                )
                .await?;
                bewaar(state, &uit, None)?;
            }
            Ok(nr.to_string())
        }
    }
}

fn bewaar(state: &AppState, uit: &std::path::Path, bij: Option<&str>) -> Result<String, String> {
    let id = state
        .opslag
        .bewaar(uit, bij)
        .map_err(|e| format!("bewaren mislukte: {e}"))?;
    let t: Value =
        serde_json::from_slice(&state.opslag.tijdlijn(&id).unwrap_or_default()).unwrap_or_default();
    Ok(t["naam"].as_str().unwrap_or(&id).to_string())
}

/// For each dossier that is not there: why (in the queue, failed). The worker
/// puts it at its transition.
fn schrijf_status(state: &AppState, id: &str) -> Result<std::path::PathBuf, String> {
    let mut status = serde_json::Map::new();
    for (nr, m) in state.opslag.mislukt() {
        status.insert(nr, json!({ "status": "mislukt", "reden": m["reden"] }));
    }
    for t in state.rij.overzicht()["wachtend"]
        .as_array()
        .into_iter()
        .flatten()
    {
        if let Some(nr) = t["dossier"].as_str() {
            status.insert(nr.into(), json!({ "status": "in de rij" }));
        }
    }
    let pad = state.opslag.tijdelijk().join(format!("{id}-status.json"));
    std::fs::write(&pad, serde_json::to_vec(&status).unwrap_or_default())
        .map_err(|e| e.to_string())?;
    Ok(pad)
}

/// Put every dossier of this law that is not there yet in the queue, newest
/// first: those most often have XML and are the richest. Returns how many.
pub fn verdiep(state: &AppState, bwb: &str) -> usize {
    let Some(k) = state.opslag.keten(bwb) else {
        return 0;
    };
    let mut overgangen: Vec<&Value> = k["overgangen"].as_array().into_iter().flatten().collect();
    overgangen.sort_by(|a, b| b["tot"].as_str().cmp(&a["tot"].as_str()));
    let mislukt = state.opslag.mislukt();
    let mut gezien = Vec::new();
    for nr in overgangen
        .iter()
        .flat_map(|o| o["dossiers"].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
    {
        if gezien.iter().any(|g| g == nr)
            || !crate::id::is_dossier(nr)
            || state.opslag.heeft(nr)
            || mislukt.contains_key(nr)
        {
            continue;
        }
        gezien.push(nr.to_string());
        state.rij.voeg(Wat::Dossier, bwb, Some(nr), Some(bwb), true);
    }
    gezien.len()
}

/// After a restart: queue again what the fetched laws still miss.
pub fn herstel(state: &AppState) -> usize {
    state
        .opslag
        .index()
        .iter()
        .filter(|d| d["soort"] == "keten" && d["opgehaald"] == true)
        .filter_map(|d| d["dossier"].as_str())
        .map(|bwb| verdiep(state, bwb))
        .sum()
}
