use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::types::{Job, JobType};

pub struct JobQueue {
    db: PgPool,
}

impl JobQueue {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn enqueue(&self, job_type: &JobType) -> Result<Uuid> {
        let payload = serde_json::to_value(job_type)?;
        let type_name = match job_type {
            JobType::SendMaintenanceReminder { .. } => "send_maintenance_reminder",
            JobType::SendWarrantyReminder { .. } => "send_warranty_reminder",
            JobType::DispatchWebhook { .. } => "dispatch_webhook",
            JobType::SendEmail { .. } => "send_email",
        };
        let id = kepta_db::jobs::enqueue(&self.db, type_name, payload).await?;
        Ok(id)
    }

    pub async fn claim_next(&self) -> Result<Option<Job>> {
        let row = kepta_db::jobs::claim_next(&self.db).await?;
        let Some(row) = row else { return Ok(None) };

        let job_type: JobType = serde_json::from_value(row.payload)?;
        Ok(Some(Job { id: row.id, job_type, attempts: row.attempts }))
    }

    pub async fn complete(&self, id: Uuid) -> Result<()> {
        kepta_db::jobs::complete(&self.db, id).await
    }

    pub async fn fail(&self, id: Uuid, error: &str) -> Result<()> {
        kepta_db::jobs::fail(&self.db, id, error).await
    }
}
