use crate::db::DbPool;
use crate::models::receipt::Receipt;
use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub struct ReceiptRepository;

impl ReceiptRepository {
    pub async fn create<'e, E>(
        executor: E,
        tenant_id: Uuid,
        rent_ledger_id: Option<Uuid>,
        payment_transaction_id: Option<Uuid>,
        payment_method: &str,
        amount: f64,
        issued_by: Option<Uuid>,
    ) -> Result<Receipt, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        let receipt_number = Self::generate_receipt_number();

        let receipt = sqlx::query_as::<_, Receipt>(
            r#"
            INSERT INTO receipts (
                receipt_number, tenant_id, rent_ledger_id, payment_transaction_id,
                payment_method, amount, issued_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, receipt_number, tenant_id, rent_ledger_id, payment_transaction_id,
                payment_method, amount, issued_at, receipt_file_url, issued_by
            "#,
        )
        .bind(&receipt_number)
        .bind(tenant_id)
        .bind(rent_ledger_id)
        .bind(payment_transaction_id)
        .bind(payment_method)
        .bind(amount)
        .bind(issued_by)
        .fetch_one(executor)
        .await?;

        Ok(receipt)
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<Receipt>, sqlx::Error> {
        let receipts = sqlx::query_as::<_, Receipt>(
            r#"
            SELECT
                id, receipt_number, tenant_id, rent_ledger_id, payment_transaction_id,
                payment_method, amount, issued_at, receipt_file_url, issued_by
            FROM receipts
            ORDER BY issued_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(receipts)
    }

    pub async fn find_by_tenant(pool: &DbPool, tenant_id: Uuid) -> Result<Vec<Receipt>, sqlx::Error> {
        let receipts = sqlx::query_as::<_, Receipt>(
            r#"
            SELECT
                id, receipt_number, tenant_id, rent_ledger_id, payment_transaction_id,
                payment_method, amount, issued_at, receipt_file_url, issued_by
            FROM receipts
            WHERE tenant_id = $1
            ORDER BY issued_at DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(pool)
        .await?;

        Ok(receipts)
    }

    pub async fn find_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Receipt>, sqlx::Error> {
        let receipt = sqlx::query_as::<_, Receipt>(
            r#"
            SELECT
                id, receipt_number, tenant_id, rent_ledger_id, payment_transaction_id,
                payment_method, amount, issued_at, receipt_file_url, issued_by
            FROM receipts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(receipt)
    }

    fn generate_receipt_number() -> String {
        let now = chrono::Utc::now();
        let year = now.format("%Y").to_string();
        let uuid_str = Uuid::new_v4().to_string();
        let suffix = uuid_str[..6].to_uppercase();
        format!("RCP-{}-{}", year, suffix)
    }
}
