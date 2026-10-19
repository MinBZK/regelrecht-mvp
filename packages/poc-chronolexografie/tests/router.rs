//! The app under its prefix, as the poc portal forwards to it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::Path;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use regelrecht_poc_chronolexografie::{api::AppState, kern::Corpus, router};
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> axum::Router {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let casussen = Corpus::alle(&dir.join("../../corpus"), &dir.join("casussen")).unwrap();
    router(
        AppState::new(casussen),
        "/chronolexografie/",
        "/nonexistent",
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

/// The app with the case `id` chosen.
async fn app_met(id: &str) -> axum::Router {
    let app = app();
    let (status, _) = send(
        &app,
        post("/chronolexografie/api/casus", json!({ "casus": id })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    app
}

async fn send(app: &axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn post(uri: &str, body: Value) -> Request<Body> {
    Request::post(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn the_api_answers_under_the_prefix() {
    let app = app_met("vakantieverlof").await;
    let (status, vormen) = send(
        &app,
        Request::get("/chronolexografie/api/vormen?datum=2026-10-14")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        vormen["aanvraag"]["grondslag"],
        json!("leerplichtwet_1969#13a")
    );

    let response = app
        .clone()
        .oneshot(
            Request::get("/chronolexografie/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
}

#[tokio::test]
async fn a_refusal_is_a_422_with_the_reason_and_reset_empties_the_chronicle() {
    let app = app_met("vakantieverlof").await;
    let (status, body) = send(
        &app,
        post(
            "/chronolexografie/api/aanvragen",
            json!({ "velden": { "bestemming": "Lissabon" }, "datum": "2026-10-13" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(
        body["fout"].as_str().unwrap().contains("bestemming"),
        "{body}"
    );

    let (status, _) = send(
        &app,
        post(
            "/chronolexografie/api/aanvragen",
            json!({ "velden": { "bsn_jongere": "999990019" }, "datum": "2026-10-13" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let kroniek = |app: axum::Router| async move {
        send(
            &app,
            Request::get("/chronolexografie/api/kroniek")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .1
    };
    assert_eq!(
        kroniek(app.clone()).await["grammen"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // The log has the application as its one action; it is no decision.
    let (status, logboek) = send(
        &app,
        Request::get("/chronolexografie/api/logboek")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let handelingen = logboek["handelingen"].as_array().unwrap();
    assert_eq!(handelingen.len(), 1);
    assert_eq!(handelingen[0]["gram"]["kind"], json!("AANVRAAG"));
    assert!(handelingen[0]["waarom"].is_null());
    send(&app, post("/chronolexografie/api/reset", json!({}))).await;
    assert!(kroniek(app.clone()).await["grammen"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn without_a_chosen_case_a_step_is_a_409_and_choosing_one_starts_an_empty_chronicle() {
    let app = app();
    let (status, lijst) = send(&app, get("/chronolexografie/api/casussen")).await;
    assert_eq!(status, StatusCode::OK);
    assert!(lijst["gekozen"].is_null());
    let ids: Vec<&str> = lijst["casussen"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["gehandicaptenparkeerkaart", "vakantieverlof"]);

    for uri in [
        "/chronolexografie/api/kroniek",
        "/chronolexografie/api/casus",
    ] {
        let (status, body) = send(&app, get(uri)).await;
        assert_eq!(status, StatusCode::CONFLICT, "{uri}");
        assert!(
            body["fout"].as_str().unwrap().contains("no case is chosen"),
            "{body}"
        );
    }
    let (status, _) = send(
        &app,
        post("/chronolexografie/api/casus", json!({ "casus": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Choosing a case gives what follows for it from the law.
    let (_, casus) = send(
        &app,
        post(
            "/chronolexografie/api/casus",
            json!({ "casus": "vakantieverlof" }),
        ),
    )
    .await;
    assert_eq!(casus["besluit"], json!("leerplichtwet_1969#13a"));
    assert_eq!(
        casus["beslissing_op_bezwaar"],
        json!("algemene_wet_bestuursrecht#7:11")
    );
    assert_eq!(casus["rol"], json!("het hoofd"));
    let (_, lijst) = send(&app, get("/chronolexografie/api/casussen")).await;
    assert_eq!(lijst["gekozen"], json!("vakantieverlof"));

    // A step that expects another case than the chosen one is refused: it
    // would land in the wrong cell.
    let verwacht = |casus: &str| {
        Request::get("/chronolexografie/api/kroniek")
            .header("x-casus", casus)
            .body(Body::empty())
            .unwrap()
    };
    let (status, body) = send(&app, verwacht("gehandicaptenparkeerkaart")).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(
        body["fout"]
            .as_str()
            .unwrap()
            .contains("the chosen case is now 'vakantieverlof'"),
        "{body}"
    );
    let (status, _) = send(&app, verwacht("vakantieverlof")).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn parkeerkaart_a_second_card_next_to_a_valid_one_through_the_api() {
    // Scenario 3 of the parkeerkaart, as the app plays it: the case's own
    // example values, two applications and two decisions.
    let app = app_met("gehandicaptenparkeerkaart").await;
    let (_, casus) = send(&app, get("/chronolexografie/api/casus")).await;
    assert_eq!(
        casus["rol"],
        json!("het college van burgemeester en wethouders")
    );
    let aanvraag = casus["voorbeeld"]["aanvraag"].clone();
    let bevindingen = casus["voorbeeld"]["bevindingen"].clone();
    let uitkomst = casus["uitkomst"].as_str().unwrap().to_string();

    let mut besluiten = Vec::new();
    for datum in ["2026-10-20", "2027-03-01"] {
        let (status, app_gram) = send(
            &app,
            post(
                "/chronolexografie/api/aanvragen",
                json!({ "velden": aanvraag, "datum": datum }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{app_gram}");
        let (status, besluit) = send(
            &app,
            post(
                "/chronolexografie/api/besluiten",
                json!({ "op": app_gram["id"], "bevindingen": bevindingen, "datum": datum }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{besluit}");
        besluiten.push(besluit);
    }
    assert_eq!(besluiten[0]["besluit"]["outputs"][&uitkomst], json!(true));
    assert_eq!(besluiten[1]["besluit"]["outputs"][&uitkomst], json!(false));
    let antwoord = &besluiten[1]["waarom"]["antwoorden"][0];
    assert_eq!(
        antwoord["begrip"],
        json!("aanvrager_heeft_geldige_gehandicaptenparkeerkaart")
    );
    assert_eq!(antwoord["waarde"], json!(true));

    let id = besluiten[1]["besluit"]["id"].as_str().unwrap();
    let (status, hoe) = send(
        &app,
        get(&format!(
            "/chronolexografie/api/besluiten/{id}/antwoord/aanvrager_heeft_geldige_gehandicaptenparkeerkaart"
        )),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let eerste = besluiten[0]["besluit"]["id"].as_str().unwrap();
    assert!(hoe["antwoorden"][0]["spoor"]
        .as_str()
        .unwrap()
        .contains(eerste));
}
