use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub rent_ledger_id: Option<Uuid>,
    pub enrollment_payment_id: Option<Uuid>,
    pub amount: f64,
    pub payment_reference: String,
    pub gateway_order_id: Option<String>,
    pub status: String, // CREATED, SUCCESS, FAILED, EXPIRED
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub payment_request_id: Uuid,
    pub tenant_id: Uuid,
    pub gateway_transaction_id: String,
    pub amount: f64,
    pub payment_method: String, // UPI, CARD, NETBANKING, OTHER
    pub gateway_status: String,
    pub paid_at: DateTime<Utc>,
}
