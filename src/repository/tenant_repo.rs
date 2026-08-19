use crate::models::tenant::{CreateTenantEnrollmentDto, Tenant, UpdateTenantDto};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct TenantRepository;

impl TenantRepository {
    pub async fn create_enrollment<'e, E>(
        executor: E,
        dto: &CreateTenantEnrollmentDto,
    ) -> Result<Tenant, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            INSERT INTO tenants (
                full_name, contact_number, email, joining_date,
                occupation_type, organization_name, room_id,
                monthly_rent, advance_amount,
                enrollment_status, joining_payment_completed, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'PENDING_PAYMENT', FALSE, 'ACTIVE')
            RETURNING
                id, tenant_id, full_name, contact_number, email, joining_date,
                occupation_type, organization_name, room_id, monthly_rent, advance_amount,
                enrollment_status, joining_payment_completed, status, activated_at,
                created_at, updated_at
            "#,
        )
        .bind(&dto.full_name)
        .bind(&dto.contact_number)
        .bind(&dto.email)
        .bind(dto.joining_date)
        .bind(&dto.occupation_type)
        .bind(&dto.organization_name)
        .bind(dto.room_id)
        .bind(dto.monthly_rent)
        .bind(dto.advance_amount)
        .fetch_one(executor)
        .await?;

        Ok(tenant)
    }

    pub async fn find_all(pool: &sqlx::PgPool) -> Result<Vec<Tenant>, sqlx::Error> {
        let tenants = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT
                id, tenant_id, full_name, contact_number, email, joining_date,
                occupation_type, organization_name, room_id, monthly_rent, advance_amount,
                enrollment_status, joining_payment_completed, status, activated_at,
                created_at, updated_at
            FROM tenants
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(tenants)
    }

    pub async fn find_by_id(pool: &sqlx::PgPool, id: Uuid) -> Result<Option<Tenant>, sqlx::Error> {
        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            SELECT
                id, tenant_id, full_name, contact_number, email, joining_date,
                occupation_type, organization_name, room_id, monthly_rent, advance_amount,
                enrollment_status, joining_payment_completed, status, activated_at,
                created_at, updated_at
            FROM tenants
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(tenant)
    }

    pub async fn activate<'e, E>(
        executor: E,
        id: Uuid,
        tenant_code: &str,
    ) -> Result<Tenant, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let tenant = sqlx::query_as::<_, Tenant>(
            r#"
            UPDATE tenants
            SET tenant_id = $1,
                enrollment_status = 'ACTIVE',
                joining_payment_completed = TRUE,
                activated_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            RETURNING
                id, tenant_id, full_name, contact_number, email, joining_date,
                occupation_type, organization_name, room_id, monthly_rent, advance_amount,
                enrollment_status, joining_payment_completed, status, activated_at,
                created_at, updated_at
            "#,
        )
        .bind(tenant_code)
        .bind(id)
        .fetch_one(executor)
        .await?;

        Ok(tenant)
    }

    pub async fn next_tenant_code(pool: &sqlx::PgPool) -> Result<String, sqlx::Error> {
        let row: (Option<i32>,) = sqlx::query_as(
            r#"
            SELECT MAX(CAST(SUBSTRING(tenant_id FROM 5) AS INT))
            FROM tenants
            WHERE tenant_id ~ '^TNT-[0-9]+$'
            "#,
        )
        .fetch_one(pool)
        .await?;

        let next = row.0.unwrap_or(1000) + 1;
        Ok(format!("TNT-{}", next))
    }

    pub async fn update(
        pool: &sqlx::PgPool,
        id: Uuid,
        dto: UpdateTenantDto,
    ) -> Result<Option<Tenant>, sqlx::Error> {
        let existing = Self::find_by_id(pool, id).await?;
        let Some(current) = existing else {
            return Ok(None);
        };

        let full_name = dto.full_name.unwrap_or(current.full_name);
        let contact_number = dto.contact_number.unwrap_or(current.contact_number);
        let email = dto.email.or(current.email);
        let occupation_type = dto.occupation_type.or(current.occupation_type);
        let organization_name = dto.organization_name.or(current.organization_name);
        let room_id = dto.room_id.unwrap_or(current.room_id);
        let monthly_rent = dto.monthly_rent.unwrap_or(current.monthly_rent);
        let advance_amount = dto.advance_amount.unwrap_or(current.advance_amount);
        let enrollment_status = dto
            .enrollment_status
            .unwrap_or(current.enrollment_status);
        let status = dto.status.unwrap_or(current.status);

        let updated = sqlx::query_as::<_, Tenant>(
            r#"
            UPDATE tenants
            SET full_name = $1, contact_number = $2, email = $3,
                occupation_type = $4, organization_name = $5, room_id = $6,
                monthly_rent = $7, advance_amount = $8,
                enrollment_status = $9, status = $10,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $11
            RETURNING
                id, tenant_id, full_name, contact_number, email, joining_date,
                occupation_type, organization_name, room_id, monthly_rent, advance_amount,
                enrollment_status, joining_payment_completed, status, activated_at,
                created_at, updated_at
            "#,
        )
        .bind(full_name)
        .bind(contact_number)
        .bind(email)
        .bind(occupation_type)
        .bind(organization_name)
        .bind(room_id)
        .bind(monthly_rent)
        .bind(advance_amount)
        .bind(enrollment_status)
        .bind(status)
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(updated)
    }
}
