use crate::models::enrollment_payment::{EnrollmentPayment, RecordEnrollmentPaymentDto};
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct EnrollmentPaymentRepository;

impl EnrollmentPaymentRepository {
    pub async fn create_for_tenant<'e, E>(
        executor: E,
        tenant_id: Uuid,
        payment_type: &str,
        amount_due: f64,
    ) -> Result<EnrollmentPayment, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let payment = sqlx::query_as::<_, EnrollmentPayment>(
            r#"
            INSERT INTO enrollment_payments (tenant_id, payment_type, amount_due, payment_method, payment_status)
            VALUES ($1, $2, $3, 'CASH', 'PENDING')
            RETURNING
                id, tenant_id, payment_type, amount_due, amount_paid, payment_method,
                payment_status, paid_at, reference_id, receipt_id, created_at
            "#,
        )
        .bind(tenant_id)
        .bind(payment_type)
        .bind(amount_due)
        .fetch_one(executor)
        .await?;

        Ok(payment)
    }

    pub async fn find_by_tenant(
        pool: &sqlx::PgPool,
        tenant_id: Uuid,
    ) -> Result<Vec<EnrollmentPayment>, sqlx::Error> {
        let payments = sqlx::query_as::<_, EnrollmentPayment>(
            r#"
            SELECT
                id, tenant_id, payment_type, amount_due, amount_paid, payment_method,
                payment_status, paid_at, reference_id, receipt_id, created_at
            FROM enrollment_payments
            WHERE tenant_id = $1
            ORDER BY payment_type ASC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(payments)
    }

    pub async fn record_payment(
        pool: &sqlx::PgPool,
        dto: &RecordEnrollmentPaymentDto,
    ) -> Result<EnrollmentPayment, sqlx::Error> {
        let status = if dto.amount_paid >= dto.amount_due {
            "PAID"
        } else {
            "PENDING"
        };

        let payment = sqlx::query_as::<_, EnrollmentPayment>(
            r#"
            UPDATE enrollment_payments
            SET amount_paid = $1,
                payment_method = $2,
                payment_status = $3,
                paid_at = CASE WHEN $3 = 'PAID' THEN CURRENT_TIMESTAMP ELSE paid_at END,
                reference_id = $4
            WHERE tenant_id = $5 AND payment_type = $6
            RETURNING
                id, tenant_id, payment_type, amount_due, amount_paid, payment_method,
                payment_status, paid_at, reference_id, receipt_id, created_at
            "#,
        )
        .bind(dto.amount_paid)
        .bind(&dto.payment_method)
        .bind(status)
        .bind(&dto.reference_id)
        .bind(dto.tenant_id)
        .bind(&dto.payment_type)
        .fetch_one(pool)
        .await?;

        Ok(payment)
    }

    pub async fn all_paid(pool: &sqlx::PgPool, tenant_id: Uuid) -> Result<bool, sqlx::Error> {
        let row: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*)::bigint,
                COUNT(*) FILTER (WHERE payment_status = 'PAID')::bigint
            FROM enrollment_payments
            WHERE tenant_id = $1
            "#,
        )
        .bind(tenant_id)
        .fetch_one(pool)
        .await?;

        Ok(row.0 >= 2 && row.0 == row.1)
    }
}
