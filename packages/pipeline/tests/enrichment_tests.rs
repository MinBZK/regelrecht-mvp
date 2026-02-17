use regelrecht_pipeline::enrichment::{
    extract_yaml_from_response, yaml_to_json, AnthropicClient, EnrichmentConfig, Enricher,
    SchemaValidator,
};
use regelrecht_pipeline::enrichment::{ArticleInput, LawContext};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn sample_article() -> ArticleInput {
    ArticleInput {
        number: "2".into(),
        text: "1. Een persoon heeft recht op zorgtoeslag indien hij:\n\
               a. de leeftijd van 18 jaar heeft bereikt;\n\
               b. verzekerd is ingevolge de Zorgverzekeringswet."
            .into(),
        url: "https://wetten.overheid.nl/BWBR0018451#Artikel2".into(),
    }
}

fn sample_context() -> LawContext {
    LawContext {
        law_id: "zorgtoeslagwet".into(),
        name: "Wet op de zorgtoeslag".into(),
        regulatory_layer: "WET".into(),
        bwb_id: Some("BWBR0018451".into()),
        url: "https://wetten.overheid.nl/BWBR0018451".into(),
        publication_date: "2005-07-07".into(),
        other_articles: vec![],
        known_regulations: vec!["awir".into(), "zorgverzekeringswet".into()],
    }
}

fn valid_machine_readable_yaml() -> &'static str {
    r#"machine_readable:
  competent_authority:
    name: "Belastingdienst/Toeslagen"
  execution:
    produces:
      legal_character: "BESCHIKKING"
      decision_type: "TOEKENNING"
    parameters:
      - name: "bsn"
        type: "string"
        required: true
      - name: "peildatum"
        type: "date"
        required: true
    input:
      - name: "geboortedatum"
        type: "date"
        source:
          output: "geboortedatum"
          description: "Geboortedatum uit BRP"
      - name: "is_verzekerd"
        type: "boolean"
        source:
          regulation: "zorgverzekeringswet"
          output: "is_verzekerd"
          parameters:
            bsn: "$bsn"
    output:
      - name: "leeftijd"
        type: "number"
      - name: "heeft_recht"
        type: "boolean"
    actions:
      - output: "leeftijd"
        operation: "SUBTRACT_DATE"
        values:
          - "$peildatum"
          - "$geboortedatum"
      - output: "heeft_recht"
        operation: "AND"
        values:
          - operation: "GREATER_THAN_OR_EQUAL"
            subject: "$leeftijd"
            value: 18
          - operation: "EQUALS"
            subject: "$is_verzekerd"
            value: true"#
}

fn anthropic_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "msg_test",
        "type": "message",
        "role": "assistant",
        "content": [
            {
                "type": "text",
                "text": content
            }
        ],
        "model": "claude-sonnet-4-20250514",
        "usage": {
            "input_tokens": 500,
            "output_tokens": 300
        }
    })
}

#[tokio::test]
async fn test_successful_enrichment_e2e() {
    let mock_server = MockServer::start().await;

    // First call: enrichment request → return valid YAML
    // Second call: reverse validation → return VALID
    let enrichment_resp = anthropic_response(valid_machine_readable_yaml());
    let validation_resp = anthropic_response("VALID");

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&enrichment_resp))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&validation_resp))
        .mount(&mock_server)
        .await;

    let config = EnrichmentConfig::builder("test-key")
        .api_base_url(mock_server.uri())
        .max_fix_iterations(3)
        .build();

    let client = AnthropicClient::new(&config).expect("client creation");
    let enricher = Enricher::new(&client, &config).expect("enricher creation");

    let article = sample_article();
    let context = sample_context();

    let result = enricher.enrich_article(&article, &context).await;
    assert!(result.is_ok(), "enrichment should succeed: {:?}", result.err());

    let result = result.expect("result");
    assert_eq!(result.article_number, "2");
    assert!(result.schema_valid);
    assert!(!result.machine_readable.is_empty());
}

#[tokio::test]
async fn test_api_error_handling() {
    let mock_server = MockServer::start().await;

    // Return a 400 error
    let error_resp = serde_json::json!({
        "error": {
            "type": "invalid_request_error",
            "message": "Invalid model specified"
        }
    });

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(400).set_body_json(&error_resp))
        .mount(&mock_server)
        .await;

    let config = EnrichmentConfig::builder("test-key")
        .api_base_url(mock_server.uri())
        .build();

    let client = AnthropicClient::new(&config).expect("client creation");
    let enricher = Enricher::new(&client, &config).expect("enricher creation");

    let result = enricher
        .enrich_article(&sample_article(), &sample_context())
        .await;

    assert!(result.is_err());
    let err_str = result.expect_err("should be an error").to_string();
    assert!(
        err_str.contains("Invalid model"),
        "Error should contain API message: {err_str}"
    );
}

#[tokio::test]
async fn test_schema_validator_with_fixture() {
    let valid_json: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/valid_machine_readable.json"))
            .expect("fixture parse");
    let invalid_json: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/invalid_machine_readable.json"))
            .expect("fixture parse");

    let validator = SchemaValidator::new().expect("validator");

    assert!(
        validator.validate(&valid_json).is_ok(),
        "valid fixture should pass"
    );
    assert!(
        validator.validate(&invalid_json).is_err(),
        "invalid fixture should fail"
    );
}

#[test]
fn test_yaml_to_json_roundtrip() {
    let yaml = valid_machine_readable_yaml();
    let json = yaml_to_json(yaml).expect("conversion");

    // Should have extracted the inner machine_readable
    assert!(json.get("competent_authority").is_some());
    assert!(json.get("execution").is_some());

    // Should pass schema validation
    let validator = SchemaValidator::new().expect("validator");
    assert!(
        validator.validate(&json).is_ok(),
        "converted YAML should pass schema: {:?}",
        validator.validate(&json).err()
    );
}

#[test]
fn test_extract_yaml_strips_fences() {
    let with_fences = "Here is the output:\n\n```yaml\nmachine_readable:\n  execution: {}\n```\n\nDone!";
    let extracted = extract_yaml_from_response(with_fences);
    assert!(
        !extracted.contains("```"),
        "fences should be stripped"
    );
    assert!(extracted.starts_with("machine_readable:"));
}

#[tokio::test]
async fn test_enrich_law_handles_failures() {
    let mock_server = MockServer::start().await;

    // Always return an error
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let config = EnrichmentConfig::builder("test-key")
        .api_base_url(mock_server.uri())
        .max_fix_iterations(1)
        .timeout_secs(5)
        .build();

    let client = AnthropicClient::new(&config).expect("client creation");
    let enricher = Enricher::new(&client, &config).expect("enricher creation");

    let articles = vec![sample_article()];
    let context = sample_context();

    let result = enricher.enrich_law(&articles, &context).await;

    assert_eq!(result.total_articles, 1);
    assert_eq!(result.failed_count, 1);
    assert_eq!(result.enriched_count, 0);
}
