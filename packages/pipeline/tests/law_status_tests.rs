mod common;

use pretty_assertions::assert_eq;

use regelrecht_pipeline::job_queue::{self, CreateJobRequest};
use regelrecht_pipeline::law_status;
use regelrecht_pipeline::models::{JobType, LawStatusValue};

#[tokio::test]
async fn test_upsert_law() {
    let db = common::TestDb::new().await;

    let entry = law_status::upsert_law(&db.pool, "zorgtoeslagwet", Some("Zorgtoeslagwet"))
        .await
        .unwrap();

    assert_eq!(entry.law_id, "zorgtoeslagwet");
    assert_eq!(entry.law_name, Some("Zorgtoeslagwet".to_string()));
    assert_eq!(entry.status, LawStatusValue::Unknown);
    assert!(entry.quality_score.is_none());

    // Upsert again with different name
    let updated = law_status::upsert_law(&db.pool, "zorgtoeslagwet", Some("Zorgtoeslagwet v2"))
        .await
        .unwrap();
    assert_eq!(updated.law_name, Some("Zorgtoeslagwet v2".to_string()));
}

#[tokio::test]
async fn test_upsert_law_without_name() {
    let db = common::TestDb::new().await;

    // Create with name
    law_status::upsert_law(&db.pool, "test_law", Some("Test Law"))
        .await
        .unwrap();

    // Upsert without name should preserve existing name
    let entry = law_status::upsert_law(&db.pool, "test_law", None)
        .await
        .unwrap();
    assert_eq!(entry.law_name, Some("Test Law".to_string()));
}

#[tokio::test]
async fn test_update_status() {
    let db = common::TestDb::new().await;

    law_status::upsert_law(&db.pool, "test_law", None)
        .await
        .unwrap();

    let entry = law_status::update_status(&db.pool, "test_law", LawStatusValue::Queued)
        .await
        .unwrap();
    assert_eq!(entry.status, LawStatusValue::Queued);

    let entry = law_status::update_status(&db.pool, "test_law", LawStatusValue::Harvesting)
        .await
        .unwrap();
    assert_eq!(entry.status, LawStatusValue::Harvesting);
}

#[tokio::test]
async fn test_update_status_not_found() {
    let db = common::TestDb::new().await;

    let result = law_status::update_status(&db.pool, "nonexistent", LawStatusValue::Queued).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_job_links() {
    let db = common::TestDb::new().await;

    law_status::upsert_law(&db.pool, "test_law", None)
        .await
        .unwrap();

    // Create a harvest job and link it
    let job = job_queue::create_job(
        &db.pool,
        CreateJobRequest::new(JobType::Harvest, "test_law"),
    )
    .await
    .unwrap();

    let entry = law_status::set_harvest_job(&db.pool, "test_law", job.id)
        .await
        .unwrap();
    assert_eq!(entry.harvest_job_id, Some(job.id));

    // Create an enrich job and link it
    let enrich_job =
        job_queue::create_job(&db.pool, CreateJobRequest::new(JobType::Enrich, "test_law"))
            .await
            .unwrap();

    let entry = law_status::set_enrich_job(&db.pool, "test_law", enrich_job.id)
        .await
        .unwrap();
    assert_eq!(entry.enrich_job_id, Some(enrich_job.id));
}

#[tokio::test]
async fn test_set_quality_score() {
    let db = common::TestDb::new().await;

    law_status::upsert_law(&db.pool, "test_law", None)
        .await
        .unwrap();

    let entry = law_status::set_quality_score(&db.pool, "test_law", 0.85)
        .await
        .unwrap();
    assert_eq!(entry.quality_score, Some(0.85));
}

#[tokio::test]
async fn test_get_law() {
    let db = common::TestDb::new().await;

    law_status::upsert_law(&db.pool, "zorgtoeslagwet", Some("Zorgtoeslagwet"))
        .await
        .unwrap();

    let entry = law_status::get_law(&db.pool, "zorgtoeslagwet")
        .await
        .unwrap();
    assert_eq!(entry.law_id, "zorgtoeslagwet");
}

#[tokio::test]
async fn test_get_law_not_found() {
    let db = common::TestDb::new().await;

    let result = law_status::get_law(&db.pool, "nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_laws() {
    let db = common::TestDb::new().await;

    law_status::upsert_law(&db.pool, "law_a", Some("Law A"))
        .await
        .unwrap();
    law_status::upsert_law(&db.pool, "law_b", Some("Law B"))
        .await
        .unwrap();

    // Update one to harvested
    law_status::update_status(&db.pool, "law_b", LawStatusValue::Harvested)
        .await
        .unwrap();

    let all = law_status::list_laws(&db.pool, None).await.unwrap();
    assert_eq!(all.len(), 2);

    let unknown = law_status::list_laws(&db.pool, Some(LawStatusValue::Unknown))
        .await
        .unwrap();
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].law_id, "law_a");

    let harvested = law_status::list_laws(&db.pool, Some(LawStatusValue::Harvested))
        .await
        .unwrap();
    assert_eq!(harvested.len(), 1);
    assert_eq!(harvested[0].law_id, "law_b");
}
