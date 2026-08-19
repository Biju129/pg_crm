use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notice {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoticeDto {
    pub title: String,
    pub content: String,
    pub created_by: Option<Uuid>,
}
