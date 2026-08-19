use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VacateRequest {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub requested_date: NaiveDate,
    pub planned_vacate_date: NaiveDate,
    pub notice_period_days: i32,
    pub status: String, // REQUESTED, UNDER_REVIEW, APPROVED, REJECTED, COMPLETED
    pub admin_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VacateSettlement {
    pub id: Uuid,
    pub vacate_request_id: Uuid,
    pub tenant_id: Uuid,
    pub advance_amount: f64,
    pub pending_rent_deduction: f64,
    pub damage_deduction: f64,
    pub other_deduction: f64,
    pub total_deduction: f64,
    pub refund_amount: f64,
    pub inspection_completed: bool,
    pub keys_returned: bool,
    pub items_returned: bool,
    pub refund_status: String, // PENDING, APPROVED, PAID
    pub refund_payment_reference: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}
