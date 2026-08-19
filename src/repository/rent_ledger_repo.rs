use crate::models::rent_ledger::RentLedger;
use chrono::NaiveDate;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct RentLedgerRepository;

impl RentLedgerRepository {
    pub async fn create<'e, E>(
        executor: E,
        tenant_id: Uuid,
        room_id: Uuid,
        billing_month: NaiveDate,
        due_date: NaiveDate,
        rent_due: f64,
    ) -> Result<RentLedger, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let pending = rent_due;

        let entry = sqlx::query_as::<_, RentLedger>(
            r#"
            INSERT INTO rent_ledger (
                tenant_id, room_id, billing_month, due_date,
                rent_due, amount_paid, pending_amount, payment_status
            )
            VALUES ($1, $2, $3, $4, $5, 0, $6, 'PENDING')
            RETURNING
                id, tenant_id, room_id, billing_month, due_date, rent_due,
                amount_paid, pending_amount, payment_status, payment_method,
                paid_at, last_reminder_sent_at, created_at, updated_at
            "#,
        )
        .bind(tenant_id)
        .bind(room_id)
        .bind(billing_month)
        .bind(due_date)
        .bind(rent_due)
        .bind(pending)
        .fetch_one(executor)
        .await?;

        Ok(entry)
    }

    pub async fn find_all(pool: &sqlx::PgPool) -> Result<Vec<RentLedger>, sqlx::Error> {
        let entries = sqlx::query_as::<_, RentLedger>(
            r#"
            SELECT
                id, tenant_id, room_id, billing_month, due_date, rent_due,
                amount_paid, pending_amount, payment_status, payment_method,
                paid_at, last_reminder_sent_at, created_at, updated_at
            FROM rent_ledger
            ORDER BY due_date DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(entries)
    }

    pub async fn find_by_tenant(
        pool: &sqlx::PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<RentLedger>, sqlx::Error> {
        let entries = sqlx::query_as::<_, RentLedger>(
            r#"
            SELECT
                id, tenant_id, room_id, billing_month, due_date, rent_due,
                amount_paid, pending_amount, payment_status, payment_method,
                paid_at, last_reminder_sent_at, created_at, updated_at
            FROM rent_ledger
            WHERE tenant_id = $1
            ORDER BY due_date DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(entries)
    }

    pub async fn find_by_id(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<Option<RentLedger>, sqlx::Error> {
        let entry = sqlx::query_as::<_, RentLedger>(
            r#"
            SELECT
                id, tenant_id, room_id, billing_month, due_date, rent_due,
                amount_paid, pending_amount, payment_status, payment_method,
                paid_at, last_reminder_sent_at, created_at, updated_at
            FROM rent_ledger
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(entry)
    }

    pub async fn record_payment<'e, E>(
        executor: E,
        id: Uuid,
        payment_amount: f64,
        payment_method: &str,
    ) -> Result<RentLedger, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let entry = sqlx::query_as::<_, RentLedger>(
            r#"
            UPDATE rent_ledger
            SET amount_paid = amount_paid + $1,
                pending_amount = GREATEST(0.0, rent_due - (amount_paid + $1)),
                payment_status = CASE
                    WHEN (rent_due - (amount_paid + $1)) <= 0 THEN 'PAID'
                    WHEN (amount_paid + $1) > 0 THEN 'PARTIAL'
                    ELSE 'PENDING'
                END,
                payment_method = $2,
                paid_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $3
            RETURNING
                id, tenant_id, room_id, billing_month, due_date, rent_due,
                amount_paid, pending_amount, payment_status, payment_method,
                paid_at, last_reminder_sent_at, created_at, updated_at
            "#,
        )
        .bind(payment_amount)
        .bind(payment_method)
        .bind(id)
        .fetch_one(executor)
        .await?;

        Ok(entry)
    }

    pub async fn update_reminder_sent(
        pool: &sqlx::PgPool,
        id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE rent_ledger SET last_reminder_sent_at = CURRENT_TIMESTAMP WHERE id = $1",
        )
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn mark_overdue_items(pool: &sqlx::PgPool) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE rent_ledger
            SET payment_status = 'OVERDUE', updated_at = CURRENT_TIMESTAMP
            WHERE due_date < CURRENT_DATE
              AND payment_status IN ('PENDING', 'PARTIAL')
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
