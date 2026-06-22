use anyhow::Result;
use kepta_jobs::{JobQueue, JobType};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,kepta_worker=debug".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let database_url = std::env::var("DATABASE_URL")?;
    let db = kepta_db::connect(&database_url).await?;
    let queue = Arc::new(JobQueue::new(db));

    tracing::info!("worker started, polling for jobs");

    loop {
        match queue.claim_next().await {
            Ok(Some(job)) => {
                tracing::info!(job_id = %job.id, job_type = ?job.job_type, "processing job");
                let result = process(&job.job_type).await;
                match result {
                    Ok(()) => {
                        queue.complete(job.id).await?;
                        tracing::info!(job_id = %job.id, "job completed");
                    }
                    Err(e) => {
                        tracing::error!(job_id = %job.id, error = %e, "job failed");
                        queue.fail(job.id, &e.to_string()).await?;
                    }
                }
            }
            Ok(None) => {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            Err(e) => {
                tracing::error!(error = %e, "error claiming job");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process(job: &JobType) -> Result<()> {
    match job {
        JobType::SendMaintenanceReminder { schedule_id, household_id } => {
            tracing::info!(%schedule_id, %household_id, "sending maintenance reminder");
            // TODO: fetch schedule, send email/push/webhook
            Ok(())
        }
        JobType::SendWarrantyReminder { item_id, household_id } => {
            tracing::info!(%item_id, %household_id, "sending warranty reminder");
            Ok(())
        }
        JobType::DispatchWebhook { webhook_id, event_type, payload } => {
            tracing::info!(%webhook_id, %event_type, "dispatching webhook");
            // TODO: fetch webhook url+secret, call kepta_notify::webhook::dispatch
            Ok(())
        }
        JobType::SendEmail { to, subject, html } => {
            tracing::info!(%to, %subject, "sending email");
            // TODO: use kepta_notify::email::EmailSender
            Ok(())
        }
    }
}
