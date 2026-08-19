use crate::db::DbPool;
use crate::models::room::{CreateRoomDto, Room, RoomResponse, UpdateRoomDto};
use crate::repository::RoomRepository;
use uuid::Uuid;

pub struct RoomService;

impl RoomService {
    pub async fn create_room(pool: &DbPool, dto: CreateRoomDto) -> Result<Room, String> {
        if dto.room_number.trim().is_empty() {
            return Err("Room number cannot be empty".to_string());
        }

        RoomRepository::create(pool, dto)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_rooms(pool: &DbPool) -> Result<Vec<RoomResponse>, String> {
        let rooms = RoomRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut responses = Vec::with_capacity(rooms.len());
        for room in rooms {
            responses.push(Self::to_response(pool, room).await?);
        }
        Ok(responses)
    }

    pub async fn get_room(pool: &DbPool, id: Uuid) -> Result<Option<RoomResponse>, String> {
        let room = RoomRepository::find_by_id(pool, id)
            .await
            .map_err(|e| e.to_string())?;

        match room {
            Some(r) => Ok(Some(Self::to_response(pool, r).await?)),
            None => Ok(None),
        }
    }

    pub async fn update_room(
        pool: &DbPool,
        id: Uuid,
        dto: UpdateRoomDto,
    ) -> Result<Option<Room>, String> {
        RoomRepository::update(pool, id, dto)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn delete_room(pool: &DbPool, id: Uuid) -> Result<bool, String> {
        RoomRepository::delete(pool, id)
            .await
            .map_err(|e| e.to_string())
    }

    async fn to_response(pool: &DbPool, room: Room) -> Result<RoomResponse, String> {
        let occupancy = RoomRepository::count_active_tenants(pool, room.id)
            .await
            .map_err(|e| e.to_string())? as i32;

        Ok(RoomResponse {
            id: room.id,
            room_number: room.room_number,
            floor_number: room.floor_number,
            capacity: room.capacity,
            current_occupancy: occupancy,
            occupied_compat: occupancy,
            available_spaces: (room.capacity - occupancy).max(0),
            monthly_rent: room.monthly_rent,
            status: room.status,
        })
    }
}
