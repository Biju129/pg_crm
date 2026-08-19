use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Receipt {
    pub id: Uuid,
    pub receipt_number: String,
    pub tenant_id: Uuid,
    pub rent_ledger_id: Option<Uuid>,
    pub payment_transaction_id: Option<Uuid>,
    pub payment_method: String,
    pub amount: f64,
    pub issued_at: DateTime<Utc>,
    pub receipt_file_url: Option<String>,
    pub issued_by: Option<Uuid>,
}
