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
}
