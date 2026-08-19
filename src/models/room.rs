use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoomStatus {
    Available,
    Full,
    Maintenance,
    Inactive,
}

impl ToString for RoomStatus {
    fn to_string(&self) -> String {
        match self {
            RoomStatus::Available => "AVAILABLE".to_string(),
            RoomStatus::Full => "FULL".to_string(),
            RoomStatus::Maintenance => "MAINTENANCE".to_string(),
            RoomStatus::Inactive => "INACTIVE".to_string(),
        }
    }
}

impl From<String> for RoomStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "FULL" => RoomStatus::Full,
            "MAINTENANCE" => RoomStatus::Maintenance,
            "INACTIVE" => RoomStatus::Inactive,
            _ => RoomStatus::Available,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Room {
    pub id: Uuid,
    pub room_number: String,
    pub floor_number: i32,
    pub capacity: i32,
    pub monthly_rent: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateRoomDto {
    pub room_number: String,
    pub floor_number: Option<i32>,
    pub capacity: Option<i32>,
    pub monthly_rent: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRoomDto {
    pub room_number: Option<String>,
    pub floor_number: Option<i32>,
    pub capacity: Option<i32>,
    pub monthly_rent: Option<f64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomResponse {
    pub id: Uuid,
    pub room_number: String,
    pub floor_number: i32,
    pub capacity: i32,
    pub current_occupancy: i32,
    #[serde(rename = "occupied")]
    pub occupied_compat: i32,
    pub available_spaces: i32,
    pub monthly_rent: f64,
    pub status: String,
}
