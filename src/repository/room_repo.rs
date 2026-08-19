use crate::db::DbPool;
use crate::models::room::{CreateRoomDto, Room, UpdateRoomDto};
use uuid::Uuid;

pub struct RoomRepository;

impl RoomRepository {
    pub async fn create(pool: &DbPool, dto: CreateRoomDto) -> Result<Room, sqlx::Error> {
        let floor_number = dto.floor_number.unwrap_or(1);
        let capacity = dto.capacity.unwrap_or(1);

        let room = sqlx::query_as::<_, Room>(
            r#"
            INSERT INTO rooms (room_number, floor_number, capacity, monthly_rent, status)
            VALUES ($1, $2, $3, $4, 'AVAILABLE')
            RETURNING id, room_number, floor_number, capacity, monthly_rent, status, created_at, updated_at
            "#,
        )
        .bind(&dto.room_number)
        .bind(floor_number)
        .bind(capacity)
        .bind(dto.monthly_rent)
        .fetch_one(pool)
        .await?;

        Ok(room)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Room>, sqlx::Error> {
        let rooms = sqlx::query_as::<_, Room>(
            r#"
            SELECT id, room_number, floor_number, capacity, monthly_rent, status, created_at, updated_at
            FROM rooms
            ORDER BY room_number ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(rooms)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Room>, sqlx::Error> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            SELECT id, room_number, floor_number, capacity, monthly_rent, status, created_at, updated_at
            FROM rooms
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(room)
    }

    pub async fn update_status<'e, E>(
        executor: E,
        id: Uuid,
        status: &str,
    ) -> Result<Option<Room>, sqlx::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let updated = sqlx::query_as::<_, Room>(
            r#"
            UPDATE rooms
            SET status = $1, updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            RETURNING id, room_number, floor_number, capacity, monthly_rent, status, created_at, updated_at
            "#,
        )
        .bind(status)
        .bind(id)
        .fetch_optional(executor)
        .await?;

        Ok(updated)
    }

    pub async fn count_active_tenants(pool: &DbPool, room_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)::bigint
            FROM tenants
            WHERE room_id = $1
              AND enrollment_status = 'ACTIVE'
              AND status = 'ACTIVE'
            "#,
        )
        .bind(room_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        dto: UpdateRoomDto,
    ) -> Result<Option<Room>, sqlx::Error> {
        let existing = Self::find_by_id(pool, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let current = existing.unwrap();

        let room_number = dto.room_number.unwrap_or(current.room_number);
        let floor_number = dto.floor_number.unwrap_or(current.floor_number);
        let capacity = dto.capacity.unwrap_or(current.capacity);
        let monthly_rent = dto.monthly_rent.unwrap_or(current.monthly_rent);
        let status = dto.status.unwrap_or(current.status);

        let updated = sqlx::query_as::<_, Room>(
            r#"
            UPDATE rooms
            SET room_number = $1, floor_number = $2, capacity = $3,
                monthly_rent = $4, status = $5, updated_at = CURRENT_TIMESTAMP
            WHERE id = $6
            RETURNING id, room_number, floor_number, capacity, monthly_rent, status, created_at, updated_at
            "#,
        )
        .bind(room_number)
        .bind(floor_number)
        .bind(capacity)
        .bind(monthly_rent)
        .bind(status)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
