use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JobType {
    SendMaintenanceReminder {
        schedule_id: Uuid,
        household_id: Uuid,
    },
    SendWarrantyReminder {
        item_id: Uuid,
        household_id: Uuid,
    },
    DispatchWebhook {
        webhook_id: Uuid,
        event_type: String,
        payload: serde_json::Value,
    },
    SendEmail {
        to: String,
        subject: String,
        html: String,
    },
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub attempts: i32,
}
