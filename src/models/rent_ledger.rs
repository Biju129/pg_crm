use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RentPaymentStatus {
    Pending,
    Partial,
    Paid,
    Overdue,
}

impl ToString for RentPaymentStatus {
    fn to_string(&self) -> String {
        match self {
            RentPaymentStatus::Pending => "PENDING".to_string(),
            RentPaymentStatus::Partial => "PARTIAL".to_string(),
            RentPaymentStatus::Paid => "PAID".to_string(),
            RentPaymentStatus::Overdue => "OVERDUE".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RentLedger {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub room_id: Uuid,
    pub billing_month: NaiveDate,
    pub due_date: NaiveDate,
    pub rent_due: f64,
    pub amount_paid: f64,
    pub pending_amount: f64,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub last_reminder_sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRentLedgerDto {
    pub tenant_id: Uuid,
    pub room_id: Uuid,
    pub billing_month: NaiveDate,
    pub due_date: NaiveDate,
    pub rent_due: f64,
}
