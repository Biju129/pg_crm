use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;

use crate::app::desktop::AppState;
use crate::models::enrollment_payment::RecordEnrollmentPaymentDto;
use crate::models::guest::{CreateGuestDto, UpdateGuestDto};
use crate::models::room::{CreateRoomDto, UpdateRoomDto};
use crate::models::tenant::CreateTenantEnrollmentDto;
use crate::models::tenant_document::UploadTenantDocumentDto;
use crate::models::user::{LoginDto, RegisterUserDto};
use crate::services::{
    ledger_service::PayRentDto, AuthService, DocumentService, GuestService, LedgerService,
    ReceiptService, ReminderEngine, RoomService, TenantService,
};

#[derive(Clone)]
pub struct ApiState {
    pub app: Arc<AppState>,
}

pub async fn start_server(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let api_state = ApiState { app: state.clone() };

    let api_routes = Router::new()
        // Auth
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        // Rooms
        .route("/rooms", get(list_rooms).post(create_room))
        .route("/rooms/:id", put(update_room).delete(delete_room))
        // Guests (legacy support)
        .route("/guests", get(list_guests).post(create_guest))
        .route("/guests/:id", get(get_guest).put(update_guest).delete(delete_guest))
        // Tenants & Enrollment
        .route("/tenants", get(list_tenants).post(enroll_tenant))
        .route("/tenants/:id", get(get_tenant))
        .route("/tenants/:id/payments", post(record_tenant_payment))
        .route("/tenants/:id/verify", post(verify_tenant))
        // Documents (Proof attachments)
        .route("/documents", post(upload_document))
        .route("/documents/tenant/:id", get(list_tenant_documents))
        .route("/documents/:id", delete(delete_document))
        // Rent Ledger
        .route("/ledger", get(list_ledger))
        .route("/ledger/pay", post(pay_rent))
        .route("/ledger/tenant/:id", get(get_tenant_ledger))
        // Receipts
        .route("/receipts", get(list_receipts))
        .route("/receipts/:id", get(get_receipt))
        .route("/receipts/tenant/:id", get(get_tenant_receipts))
        // Reminder Engine & Notifications
        .route("/reminders/run", post(run_reminders))
        .route("/notifications", get(list_notifications))
        .with_state(api_state);

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("ui"))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = format!("0.0.0.0:{}", state.config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("HTTP server listening on http://localhost:{}", state.config.app_port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn register(
    State(state): State<ApiState>,
    Json(mut dto): Json<RegisterUserDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if dto.username_or_login.is_empty() {
        if let Some(email) = dto.email.take() {
            dto.username_or_login = email;
        }
    }
    let auth = AuthService::register(
        &state.app.pool,
        dto,
        &state.app.config.jwt_secret,
        state.app.config.jwt_expires_in,
    )
    .await?;
    Ok(Json(serde_json::to_value(auth).unwrap()))
}

async fn login(
    State(state): State<ApiState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let auth = AuthService::login(
        &state.app.pool,
        dto,
        &state.app.config.jwt_secret,
        state.app.config.jwt_expires_in,
    )
    .await?;
    Ok(Json(serde_json::to_value(auth).unwrap()))
}

async fn list_rooms(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let rooms = RoomService::list_rooms(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(rooms).unwrap()))
}

async fn create_room(
    State(state): State<ApiState>,
    Json(dto): Json<CreateRoomDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let room = RoomService::create_room(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(room).unwrap()))
}

async fn update_room(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateRoomDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let room = RoomService::update_room(&state.app.pool, id, dto).await?;
    Ok(Json(serde_json::json!({ "updated": room.is_some(), "room": room })))
}

async fn delete_room(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let success = RoomService::delete_room(&state.app.pool, id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

async fn list_guests(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let guests = GuestService::list_guests(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(guests).unwrap()))
}

async fn create_guest(
    State(state): State<ApiState>,
    Json(dto): Json<CreateGuestDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guest = GuestService::create_guest(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(guest).unwrap()))
}

async fn get_guest(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guest = GuestService::get_guest(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(guest).unwrap()))
}

async fn update_guest(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateGuestDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guest = GuestService::update_guest(&state.app.pool, id, dto).await?;
    Ok(Json(serde_json::to_value(guest).unwrap()))
}

async fn delete_guest(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let success = GuestService::delete_guest(&state.app.pool, id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

async fn list_tenants(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let tenants = TenantService::list_tenants(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(tenants).unwrap()))
}

async fn enroll_tenant(
    State(state): State<ApiState>,
    Json(dto): Json<CreateTenantEnrollmentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let detail = TenantService::create_enrollment(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}

async fn get_tenant(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let detail = TenantService::get_tenant_detail(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}

async fn record_tenant_payment(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(mut dto): Json<RecordEnrollmentPaymentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    dto.tenant_id = id;
    let payment = TenantService::record_enrollment_payment(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(payment).unwrap()))
}

async fn verify_tenant(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let result = TenantService::verify_and_activate(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

async fn upload_document(
    State(state): State<ApiState>,
    Json(dto): Json<UploadTenantDocumentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let doc = DocumentService::upload_document(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(doc).unwrap()))
}

async fn list_tenant_documents(
    State(state): State<ApiState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let docs = DocumentService::list_tenant_documents(&state.app.pool, tenant_id).await?;
    Ok(Json(serde_json::to_value(docs).unwrap()))
}

async fn delete_document(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let success = DocumentService::delete_document(&state.app.pool, id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

async fn list_ledger(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let ledger = LedgerService::list_all_ledgers(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(ledger).unwrap()))
}

async fn pay_rent(
    State(state): State<ApiState>,
    Json(dto): Json<PayRentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let res = LedgerService::pay_rent(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(res).unwrap()))
}

async fn get_tenant_ledger(
    State(state): State<ApiState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ledger = LedgerService::get_tenant_ledgers(&state.app.pool, tenant_id).await?;
    Ok(Json(serde_json::to_value(ledger).unwrap()))
}

async fn list_receipts(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let receipts = ReceiptService::list_receipts(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(receipts).unwrap()))
}

async fn get_receipt(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let receipt = ReceiptService::get_receipt(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(receipt).unwrap()))
}

async fn get_tenant_receipts(
    State(state): State<ApiState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let receipts = ReceiptService::get_tenant_receipts(&state.app.pool, tenant_id).await?;
    Ok(Json(serde_json::to_value(receipts).unwrap()))
}

async fn run_reminders(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let summary = ReminderEngine::run_reminder_cycle(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(summary).unwrap()))
}

async fn list_notifications(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let notifications = ReminderEngine::list_notifications(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(notifications).unwrap()))
}

struct ApiError(String);

impl From<String> for ApiError {
    fn from(value: String) -> Self {
        ApiError(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": self.0 })),
        )
            .into_response()
    }
}
