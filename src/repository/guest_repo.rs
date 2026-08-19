use crate::db::DbPool;
use crate::models::guest::{CreateGuestDto, Guest, UpdateGuestDto};
use chrono::Utc;
use uuid::Uuid;

pub struct GuestRepository;

impl GuestRepository {
    pub async fn create(pool: &DbPool, dto: CreateGuestDto) -> Result<Guest, sqlx::Error> {
        let check_in = dto.check_in_date.unwrap_or_else(Utc::now);
        let monthly_rent = dto.monthly_rent.unwrap_or(0.0);
        let advance_amount = dto.advance_amount.unwrap_or(0.0);
        let amount_due = dto.amount_due.unwrap_or(0.0);

        let guest = sqlx::query_as::<_, Guest>(
            r#"
            INSERT INTO guests (name, room_number, phone, check_in_date, check_out_date, monthly_rent, advance_amount, amount_due)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, room_number, phone, check_in_date, check_out_date, monthly_rent, advance_amount, amount_due, created_at, updated_at
            "#,
        )
        .bind(&dto.name)
        .bind(&dto.room_number)
        .bind(&dto.phone)
        .bind(check_in)
        .bind(dto.check_out_date)
        .bind(monthly_rent)
        .bind(advance_amount)
        .bind(amount_due)
        .fetch_one(pool)
        .await?;

        Ok(guest)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Guest>, sqlx::Error> {
        let guests = sqlx::query_as::<_, Guest>(
            r#"
            SELECT id, name, room_number, phone, check_in_date, check_out_date, monthly_rent, advance_amount, amount_due, created_at, updated_at
            FROM guests
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(guests)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Guest>, sqlx::Error> {
        let guest = sqlx::query_as::<_, Guest>(
            r#"
            SELECT id, name, room_number, phone, check_in_date, check_out_date, monthly_rent, advance_amount, amount_due, created_at, updated_at
            FROM guests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(guest)
    }

    pub async fn update(
        pool: &DbPool,
        id: Uuid,
        dto: UpdateGuestDto,
    ) -> Result<Option<Guest>, sqlx::Error> {
        let existing = Self::find_by_id(pool, id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let current = existing.unwrap();

        let name = dto.name.unwrap_or(current.name);
        let room_number = dto.room_number.unwrap_or(current.room_number);
        let phone = dto.phone.or(current.phone);
        let check_in_date = dto.check_in_date.unwrap_or(current.check_in_date);
        let check_out_date = dto.check_out_date.or(current.check_out_date);
        let monthly_rent = dto.monthly_rent.unwrap_or(current.monthly_rent);
        let advance_amount = dto.advance_amount.unwrap_or(current.advance_amount);
        let amount_due = dto.amount_due.unwrap_or(current.amount_due);

        let updated = sqlx::query_as::<_, Guest>(
            r#"
            UPDATE guests
            SET name = $1, room_number = $2, phone = $3, check_in_date = $4, check_out_date = $5,
                monthly_rent = $6, advance_amount = $7, amount_due = $8, updated_at = CURRENT_TIMESTAMP
            WHERE id = $9
            RETURNING id, name, room_number, phone, check_in_date, check_out_date, monthly_rent, advance_amount, amount_due, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(room_number)
        .bind(phone)
        .bind(check_in_date)
        .bind(check_out_date)
        .bind(monthly_rent)
        .bind(advance_amount)
        .bind(amount_due)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(updated)
    }

    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM guests WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
