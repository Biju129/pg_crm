use crate::db::DbPool;
use crate::models::enrollment_payment::{EnrollmentPayment, RecordEnrollmentPaymentDto};
use crate::models::tenant::{
    CreateTenantEnrollmentDto, TenantEnrollmentDetail, TenantResponse,
};
use crate::repository::{
    EnrollmentPaymentRepository, NotificationRepository, RentLedgerRepository, RoomRepository,
    TenantRepository, UserRepository,
};
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2, PasswordHasher,
};
use chrono::{Datelike, Months, NaiveDate};
use uuid::Uuid;

pub struct TenantService;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ActivationResult {
    pub tenant: TenantResponse,
    pub login_username: String,
    pub temporary_password: String,
    pub welcome_message_queued: bool,
}

impl TenantService {
    pub async fn create_enrollment(
        pool: &DbPool,
        dto: CreateTenantEnrollmentDto,
    ) -> Result<TenantEnrollmentDetail, String> {
        if dto.full_name.trim().is_empty() {
            return Err("Tenant name is required".to_string());
        }
        if dto.contact_number.trim().is_empty() {
            return Err("Contact number is required".to_string());
        }
        if dto.monthly_rent <= 0.0 {
            return Err("Monthly rent must be greater than zero".to_string());
        }
        if dto.advance_amount < 0.0 {
            return Err("Advance amount cannot be negative".to_string());
        }

        let room = RoomRepository::find_by_id(pool, dto.room_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Room not found".to_string())?;

        if room.status != "AVAILABLE" {
            return Err(format!(
                "Room {} is not available (status: {})",
                room.room_number, room.status
            ));
        }

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let tenant = TenantRepository::create_enrollment(&mut *tx, &dto)
            .await
            .map_err(|e| e.to_string())?;

        let advance = EnrollmentPaymentRepository::create_for_tenant(
            &mut *tx,
            tenant.id,
            "ADVANCE",
            dto.advance_amount,
        )
        .await
        .map_err(|e| e.to_string())?;

        let first_rent = EnrollmentPaymentRepository::create_for_tenant(
            &mut *tx,
            tenant.id,
            "FIRST_MONTH_RENT",
            dto.monthly_rent,
        )
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        Ok(TenantEnrollmentDetail {
            tenant: TenantResponse::from(tenant),
            enrollment_payments: vec![advance, first_rent],
        })
    }

    pub async fn list_tenants(pool: &DbPool) -> Result<Vec<TenantResponse>, String> {
        let tenants = TenantRepository::find_all(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(tenants.into_iter().map(TenantResponse::from).collect())
    }

    pub async fn get_tenant_detail(
        pool: &DbPool,
        id: Uuid,
    ) -> Result<TenantEnrollmentDetail, String> {
        let tenant = TenantRepository::find_by_id(pool, id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Tenant not found".to_string())?;

        let payments = EnrollmentPaymentRepository::find_by_tenant(pool, id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(TenantEnrollmentDetail {
            tenant: TenantResponse::from(tenant),
            enrollment_payments: payments,
        })
    }

    pub async fn record_enrollment_payment(
        pool: &DbPool,
        dto: RecordEnrollmentPaymentDto,
    ) -> Result<EnrollmentPayment, String> {
        let tenant = TenantRepository::find_by_id(pool, dto.tenant_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Tenant not found".to_string())?;

        if tenant.enrollment_status != "PENDING_PAYMENT" {
            return Err("Tenant is not awaiting enrollment payment".to_string());
        }

        if dto.payment_type != "ADVANCE" && dto.payment_type != "FIRST_MONTH_RENT" {
            return Err("Payment type must be ADVANCE or FIRST_MONTH_RENT".to_string());
        }

        if dto.payment_method != "CASH" && dto.payment_method != "ONLINE" {
            return Err("Payment method must be CASH or ONLINE".to_string());
        }

        EnrollmentPaymentRepository::record_payment(pool, &dto)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn verify_and_activate(
        pool: &DbPool,
        tenant_uuid: Uuid,
    ) -> Result<ActivationResult, String> {
        let tenant = TenantRepository::find_by_id(pool, tenant_uuid)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Tenant not found".to_string())?;

        if tenant.enrollment_status == "ACTIVE" {
            return Err("Tenant is already active".to_string());
        }

        if tenant.enrollment_status == "CANCELLED" {
            return Err("Cannot activate a cancelled enrollment".to_string());
        }

        let all_paid = EnrollmentPaymentRepository::all_paid(pool, tenant_uuid)
            .await
            .map_err(|e| e.to_string())?;

        if !all_paid {
            return Err(
                "Both ADVANCE and FIRST_MONTH_RENT payments must be PAID before activation"
                    .to_string(),
            );
        }

        let room = RoomRepository::find_by_id(pool, tenant.room_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Assigned room not found".to_string())?;

        let tenant_code = TenantRepository::next_tenant_code(pool)
            .await
            .map_err(|e| e.to_string())?;

        let (login_username, temp_password) = Self::generate_credentials(&tenant_code);
        let password_hash = Self::hash_password(&temp_password)?;

        let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

        let activated = TenantRepository::activate(&mut *tx, tenant_uuid, &tenant_code)
            .await
            .map_err(|e| e.to_string())?;

        UserRepository::create(
            &mut *tx,
            &login_username,
            &password_hash,
            "TENANT",
            Some(tenant_uuid),
            true,
        )
        .await
        .map_err(|e| e.to_string())?;

        RoomRepository::update_status(&mut *tx, tenant.room_id, "FULL")
            .await
            .map_err(|e| e.to_string())?;

        let (billing_month, due_date) = Self::compute_next_rent_due(tenant.joining_date);

        RentLedgerRepository::create(
            &mut *tx,
            tenant_uuid,
            tenant.room_id,
            billing_month,
            due_date,
            tenant.monthly_rent,
        )
        .await
        .map_err(|e| e.to_string())?;

        let welcome_ref = format!("WELCOME:{}:{}", tenant_code, tenant.contact_number);

        NotificationRepository::queue(
            &mut *tx,
            tenant_uuid,
            "WELCOME",
            "WHATSAPP",
            Some(&welcome_ref),
        )
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;

        tracing::info!(
            "Tenant activated: {} in room {} — login: {}",
            tenant_code,
            room.room_number,
            login_username
        );

        Ok(ActivationResult {
            tenant: TenantResponse::from(activated),
            login_username,
            temporary_password: temp_password,
            welcome_message_queued: true,
        })
    }

    fn compute_next_rent_due(joining_date: NaiveDate) -> (NaiveDate, NaiveDate) {
        let first_of_join_month =
            NaiveDate::from_ymd_opt(joining_date.year(), joining_date.month(), 1).unwrap();

        let billing_month = first_of_join_month
            .checked_add_months(Months::new(1))
            .unwrap();

        let due_date = NaiveDate::from_ymd_opt(
            billing_month.year(),
            billing_month.month(),
            7,
        )
        .unwrap();

        (billing_month, due_date)
    }

    fn generate_credentials(tenant_code: &str) -> (String, String) {
        let login_username = tenant_code.to_lowercase();
        let mut bytes = [0u8; 8];
        argon2::password_hash::rand_core::OsRng.fill_bytes(&mut bytes);
        let temp_password: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        (login_username, temp_password[..12].to_string())
    }

    fn hash_password(password: &str) -> Result<String, String> {
        let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| e.to_string())
    }
}
