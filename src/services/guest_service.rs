use crate::db::DbPool;
use crate::models::guest::{CreateGuestDto, Guest, UpdateGuestDto};
use crate::repository::GuestRepository;
use uuid::Uuid;

pub struct GuestService;

impl GuestService {
    pub async fn create_guest(pool: &DbPool, dto: CreateGuestDto) -> Result<Guest, String> {
        if dto.name.trim().is_empty() {
            return Err("Guest name cannot be empty".to_string());
        }
        if dto.room_number.trim().is_empty() {
            return Err("Room number cannot be empty".to_string());
        }

        GuestRepository::create(pool, dto)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_guests(pool: &DbPool) -> Result<Vec<Guest>, String> {
        GuestRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_guest(pool: &DbPool, id: Uuid) -> Result<Option<Guest>, String> {
        GuestRepository::find_by_id(pool, id)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_guest(
        pool: &DbPool,
        id: Uuid,
        dto: UpdateGuestDto,
    ) -> Result<Option<Guest>, String> {
        GuestRepository::update(pool, id, dto)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_guest(pool: &DbPool, id: Uuid) -> Result<bool, String> {
        GuestRepository::delete(pool, id)
            .await
            .map_err(|e| e.to_string())
    }
}
