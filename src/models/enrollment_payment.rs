use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnrollmentPaymentType {
    Advance,
    FirstMonthRent,
}

impl ToString for EnrollmentPaymentType {
    fn to_string(&self) -> String {
        match self {
            EnrollmentPaymentType::Advance => "ADVANCE".to_string(),
            EnrollmentPaymentType::FirstMonthRent => "FIRST_MONTH_RENT".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EnrollmentPayment {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub payment_type: String,
    pub amount_due: f64,
    pub amount_paid: f64,
    pub payment_method: String,
    pub payment_status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub reference_id: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateEnrollmentPaymentDto {
    pub tenant_id: Uuid,
    pub payment_type: String,
    pub amount_due: f64,
    pub payment_method: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordEnrollmentPaymentDto {
    pub tenant_id: Uuid,
    pub payment_type: String,
    pub amount_due: f64,
    pub amount_paid: f64,
    pub payment_method: String,
    pub reference_id: Option<String>,
}
