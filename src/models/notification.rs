use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationType {
    Welcome,
    RentDue,
    RentReminder,
    PaymentSuccess,
}

impl ToString for NotificationType {
    fn to_string(&self) -> String {
        match self {
            NotificationType::Welcome => "WELCOME".to_string(),
            NotificationType::RentDue => "RENT_DUE".to_string(),
            NotificationType::RentReminder => "RENT_REMINDER".to_string(),
            NotificationType::PaymentSuccess => "PAYMENT_SUCCESS".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationChannel {
    Whatsapp,
    Sms,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub notification_type: String,
    pub channel: String,
    pub message_reference: Option<String>,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
