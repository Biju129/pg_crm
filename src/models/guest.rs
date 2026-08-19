use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Guest {
    pub id: Uuid,
    pub name: String,
    pub room_number: String,
    pub phone: Option<String>,
    pub check_in_date: DateTime<Utc>,
    pub check_out_date: Option<DateTime<Utc>>,
    pub monthly_rent: f64,
    pub advance_amount: f64,
    pub amount_due: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGuestDto {
    pub name: String,
    pub room_number: String,
    pub phone: Option<String>,
    pub check_in_date: Option<DateTime<Utc>>,
    pub check_out_date: Option<DateTime<Utc>>,
    pub monthly_rent: Option<f64>,
    pub advance_amount: Option<f64>,
    pub amount_due: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateGuestDto {
    pub name: Option<String>,
    pub room_number: Option<String>,
    pub phone: Option<String>,
    pub check_in_date: Option<DateTime<Utc>>,
    pub check_out_date: Option<DateTime<Utc>>,
    pub monthly_rent: Option<f64>,
    pub advance_amount: Option<f64>,
    pub amount_due: Option<f64>,
}
