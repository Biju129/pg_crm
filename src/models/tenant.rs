use crate::models::enrollment_payment::EnrollmentPayment;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnrollmentStatus {
    PendingPayment,
    Active,
    Cancelled,
}

impl ToString for EnrollmentStatus {
    fn to_string(&self) -> String {
        match self {
            EnrollmentStatus::PendingPayment => "PENDING_PAYMENT".to_string(),
            EnrollmentStatus::Active => "ACTIVE".to_string(),
            EnrollmentStatus::Cancelled => "CANCELLED".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TenantLifecycleStatus {
    Active,
    VacateRequested,
    Vacated,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub tenant_id: Option<String>,
    pub full_name: String,
    pub contact_number: String,
    pub email: Option<String>,
    pub joining_date: NaiveDate,
    pub occupation_type: Option<String>,
    pub organization_name: Option<String>,
    pub room_id: Uuid,
    pub monthly_rent: f64,
    pub advance_amount: f64,
    pub enrollment_status: String,
    pub joining_payment_completed: bool,
    pub status: String,
    pub activated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTenantEnrollmentDto {
    pub full_name: String,
    pub contact_number: String,
    pub email: Option<String>,
    pub joining_date: NaiveDate,
    pub occupation_type: Option<String>,
    pub organization_name: Option<String>,
    pub room_id: Uuid,
    pub monthly_rent: f64,
    pub advance_amount: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTenantDto {
    pub full_name: Option<String>,
    pub contact_number: Option<String>,
    pub email: Option<String>,
    pub occupation_type: Option<String>,
    pub organization_name: Option<String>,
    pub room_id: Option<Uuid>,
    pub monthly_rent: Option<f64>,
    pub advance_amount: Option<f64>,
    pub enrollment_status: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub tenant_id: Option<String>,
    pub full_name: String,
    pub contact_number: String,
    pub email: Option<String>,
    pub joining_date: NaiveDate,
    pub occupation_type: Option<String>,
    pub organization_name: Option<String>,
    pub room_id: Uuid,
    pub monthly_rent: f64,
    pub advance_amount: f64,
    pub total_initial_payable: f64,
    pub enrollment_status: String,
    pub joining_payment_completed: bool,
    pub status: String,
    pub activated_at: Option<DateTime<Utc>>,
}

impl From<Tenant> for TenantResponse {
    fn from(t: Tenant) -> Self {
        let total_initial_payable = t.advance_amount + t.monthly_rent;
        Self {
            id: t.id,
            tenant_id: t.tenant_id,
            full_name: t.full_name,
            contact_number: t.contact_number,
            email: t.email,
            joining_date: t.joining_date,
            occupation_type: t.occupation_type,
            organization_name: t.organization_name,
            room_id: t.room_id,
            monthly_rent: t.monthly_rent,
            advance_amount: t.advance_amount,
            total_initial_payable,
            enrollment_status: t.enrollment_status,
            joining_payment_completed: t.joining_payment_completed,
            status: t.status,
            activated_at: t.activated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantEnrollmentDetail {
    pub tenant: TenantResponse,
    pub enrollment_payments: Vec<EnrollmentPayment>,
}
