use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentType {
    InitialEnrollment,
    MonthlyRent,
    AdvanceOnly,
    RentOnly,
}

impl ToString for PaymentType {
    fn to_string(&self) -> String {
        match self {
            PaymentType::InitialEnrollment => "INITIAL_ENROLLMENT".to_string(),
            PaymentType::MonthlyRent => "MONTHLY_RENT".to_string(),
            PaymentType::AdvanceOnly => "ADVANCE_ONLY".to_string(),
            PaymentType::RentOnly => "RENT_ONLY".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMode {
    OnlineUpi,
    OnlineCard,
    OnlineNetbanking,
    Cash,
    BankTransfer,
}

impl ToString for PaymentMode {
    fn to_string(&self) -> String {
        match self {
            PaymentMode::OnlineUpi => "ONLINE_UPI".to_string(),
            PaymentMode::OnlineCard => "ONLINE_CARD".to_string(),
            PaymentMode::OnlineNetbanking => "ONLINE_NETBANKING".to_string(),
            PaymentMode::Cash => "CASH".to_string(),
            PaymentMode::BankTransfer => "BANK_TRANSFER".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub payment_type: String,
    pub advance_amount_paid: f64,
    pub rent_amount_paid: f64,
    pub total_amount_paid: f64,
    pub payment_mode: String,
    pub transaction_ref: Option<String>,
    pub verification_status: String,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub receipt_number: Option<String>,
    pub payment_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordPaymentDto {
    pub tenant_id: Uuid,
    pub payment_type: String,
    pub advance_amount_paid: Option<f64>,
    pub rent_amount_paid: Option<f64>,
    pub total_amount_paid: f64,
    pub payment_mode: String,
    pub transaction_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyPaymentDto {
    pub payment_id: Uuid,
    pub verification_status: String, // VERIFIED or REJECTED
    pub verified_by: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub payment_type: String,
    pub advance_amount_paid: f64,
    pub rent_amount_paid: f64,
    pub total_amount_paid: f64,
    pub payment_mode: String,
    pub transaction_ref: Option<String>,
    pub verification_status: String,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub receipt_number: Option<String>,
    pub payment_date: DateTime<Utc>,
}

impl From<Payment> for PaymentResponse {
    fn from(p: Payment) -> Self {
        Self {
            id: p.id,
            tenant_id: p.tenant_id,
            payment_type: p.payment_type,
            advance_amount_paid: p.advance_amount_paid,
            rent_amount_paid: p.rent_amount_paid,
            total_amount_paid: p.total_amount_paid,
            payment_mode: p.payment_mode,
            transaction_ref: p.transaction_ref,
            verification_status: p.verification_status,
            verified_by: p.verified_by,
            verified_at: p.verified_at,
            receipt_number: p.receipt_number,
            payment_date: p.payment_date,
        }
    }
}
