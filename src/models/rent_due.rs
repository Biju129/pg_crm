use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DueStatus {
    AwaitingPayment,
    DueToday,
    Overdue,
    Paid,
    PartiallyPaid,
}

impl ToString for DueStatus {
    fn to_string(&self) -> String {
        match self {
            DueStatus::AwaitingPayment => "AWAITING_PAYMENT".to_string(),
            DueStatus::DueToday => "DUE_TODAY".to_string(),
            DueStatus::Overdue => "OVERDUE".to_string(),
            DueStatus::Paid => "PAID".to_string(),
            DueStatus::PartiallyPaid => "PARTIALLY_PAID".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RentDue {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub billing_month: NaiveDate,
    pub due_date: NaiveDate,
    pub rent_amount: f64,
    pub amount_paid: f64,
    pub status: String,
    pub payment_id: Option<Uuid>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRentDueDto {
    pub tenant_id: Uuid,
    pub billing_month: NaiveDate,
    pub due_date: NaiveDate,
    pub rent_amount: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RentDueResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub billing_month: NaiveDate,
    pub due_date: NaiveDate,
    pub rent_amount: f64,
    pub amount_paid: f64,
    pub remaining_balance: f64,
    pub status: String,
    pub payment_id: Option<Uuid>,
    pub paid_at: Option<DateTime<Utc>>,
}

impl From<RentDue> for RentDueResponse {
    fn from(rd: RentDue) -> Self {
        let remaining_balance = (rd.rent_amount - rd.amount_paid).max(0.0);
        Self {
            id: rd.id,
            tenant_id: rd.tenant_id,
            billing_month: rd.billing_month,
            due_date: rd.due_date,
            rent_amount: rd.rent_amount,
            amount_paid: rd.amount_paid,
            remaining_balance,
            status: rd.status,
            payment_id: rd.payment_id,
            paid_at: rd.paid_at,
        }
    }
}
