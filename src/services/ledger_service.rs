use crate::db::DbPool;
use crate::models::receipt::Receipt;
use crate::models::rent_ledger::RentLedger;
use crate::repository::{
    NotificationRepository, ReceiptRepository, RentLedgerRepository, TenantRepository,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct LedgerService;

#[derive(Debug, Clone, Deserialize)]
pub struct PayRentDto {
    pub ledger_id: Uuid,
    pub amount_paid: f64,
    pub payment_method: String, // CASH or ONLINE
    pub issued_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PayRentResponse {
    pub ledger: RentLedger,
    pub receipt: Receipt,
    pub payment_notification_queued: bool,
}

impl LedgerService {
    pub async fn list_all_ledgers(pool: &DbPool) -> Result<Vec<RentLedger>, String> {
        RentLedgerRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_tenant_ledgers(
        pool: &DbPool,
        tenant_id: Uuid,
    ) -> Result<Vec<RentLedger>, String> {
        RentLedgerRepository::find_by_tenant(pool, tenant_id)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn pay_rent(pool: &DbPool, dto: PayRentDto) -> Result<PayRentResponse, String> {
        if dto.amount_paid <= 0.0 {
            return Err("Payment amount must be greater than zero".to_string());
        }
        if dto.payment_method != "CASH" && dto.payment_method != "ONLINE" {
            return Err("Payment method must be CASH or ONLINE".to_string());
        }

        let existing_ledger = RentLedgerRepository::find_by_id(pool, dto.ledger_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Rent ledger item not found".to_string())?;

        if existing_ledger.payment_status == "PAID" {
            return Err("This rent ledger item has already been fully paid".to_string());
        }

        let tenant = TenantRepository::find_by_id(pool, existing_ledger.tenant_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Tenant not found".to_string())?;

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let updated_ledger = RentLedgerRepository::record_payment(
            &mut *tx,
            dto.ledger_id,
            dto.amount_paid,
            &dto.payment_method,
        )
        .await
        .map_err(|e| e.to_string())?;

        let receipt = ReceiptRepository::create(
            &mut *tx,
            existing_ledger.tenant_id,
            Some(dto.ledger_id),
            None,
            &dto.payment_method,
            dto.amount_paid,
            dto.issued_by,
        )
        .await
        .map_err(|e| e.to_string())?;

        let message_ref = format!(
            "PAYMENT_CONFIRM:{}:{}:₹{}",
            receipt.receipt_number,
            tenant.tenant_id.as_deref().unwrap_or("TNT"),
            dto.amount_paid
        );

        NotificationRepository::queue(
            &mut *tx,
            existing_ledger.tenant_id,
            "PAYMENT_SUCCESS",
            "WHATSAPP",
            Some(&message_ref),
        )
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(PayRentResponse {
            ledger: updated_ledger,
            receipt,
            payment_notification_queued: true,
        })
    }
}
