use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,      // CREATE, UPDATE, PAYMENT_CONFIRM, VACATE_APPROVE, etc.
    pub entity_type: String, // TENANT, ROOM, RENT, SETTLEMENT, etc.
    pub entity_id: Option<Uuid>,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
