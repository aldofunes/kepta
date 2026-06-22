mod common;

use kepta_db::Db;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn claim_next_returns_none_when_queue_empty(db: PgPool) {
    let job = kepta_db::jobs::claim_next(&db).await.unwrap();
    assert!(job.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn enqueue_then_claim(db: PgPool) {
    let payload = json!({"type": "send_email", "to": "test@example.com"});
    let id = kepta_db::jobs::enqueue(&db, "send_email", payload.clone())
        .await
        .unwrap();

    let job = kepta_db::jobs::claim_next(&db).await.unwrap().unwrap();
    assert_eq!(job.id, id);
    assert_eq!(job.job_type, "send_email");
    assert_eq!(job.status, "processing");
    assert_eq!(job.attempts, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn claimed_job_not_visible_to_second_claimer(db: PgPool) {
    kepta_db::jobs::enqueue(&db, "send_email", json!({})).await.unwrap();
    let _first = kepta_db::jobs::claim_next(&db).await.unwrap().unwrap();
    let second = kepta_db::jobs::claim_next(&db).await.unwrap();
    assert!(second.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn complete_removes_job_from_pending(db: PgPool) {
    let id = kepta_db::jobs::enqueue(&db, "job", json!({})).await.unwrap();
    let job = kepta_db::jobs::claim_next(&db).await.unwrap().unwrap();
    kepta_db::jobs::complete(&db, job.id).await.unwrap();

    // Completing it means it won't show up again
    let next = kepta_db::jobs::claim_next(&db).await.unwrap();
    assert!(next.is_none());

    let status = sqlx::query_scalar!("SELECT status FROM jobs WHERE id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(status, "completed");
}

#[sqlx::test(migrations = "./migrations")]
async fn fail_below_max_attempts_reschedules(db: PgPool) {
    let id = kepta_db::jobs::enqueue(&db, "job", json!({})).await.unwrap();
    let job = kepta_db::jobs::claim_next(&db).await.unwrap().unwrap();
    kepta_db::jobs::fail(&db, job.id, "transient error").await.unwrap();

    let status = sqlx::query_scalar!("SELECT status FROM jobs WHERE id = $1", id)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(status, "pending");
}

#[sqlx::test(migrations = "./migrations")]
async fn jobs_claimed_in_enqueue_order(db: PgPool) {
    let id_a = kepta_db::jobs::enqueue(&db, "job_a", json!({})).await.unwrap();
    let id_b = kepta_db::jobs::enqueue(&db, "job_b", json!({})).await.unwrap();

    let first = kepta_db::jobs::claim_next(&db).await.unwrap().unwrap();
    assert_eq!(first.id, id_a);

    kepta_db::jobs::complete(&db, first.id).await.unwrap();
    let second = kepta_db::jobs::claim_next(&db).await.unwrap().unwrap();
    assert_eq!(second.id, id_b);
}
