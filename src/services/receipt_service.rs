use crate::db::DbPool;
use crate::models::receipt::Receipt;
use crate::repository::ReceiptRepository;
use uuid::Uuid;

pub struct ReceiptService;

impl ReceiptService {
    pub async fn list_receipts(pool: &DbPool) -> Result<Vec<Receipt>, String> {
        ReceiptRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_tenant_receipts(
        pool: &DbPool,
        tenant_id: Uuid,
    ) -> Result<Vec<Receipt>, String> {
        ReceiptRepository::find_by_tenant(pool, tenant_id)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_receipt(pool: &DbPool, id: Uuid) -> Result<Receipt, String> {
        ReceiptRepository::find_by_id(pool, id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Receipt not found".to_string())
    }
}
