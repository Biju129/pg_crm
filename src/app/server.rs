// ─────────────────────────────────────────────────────────────────────────────
// src/app/server.rs
// This file sets up the HTTP API server using the Axum framework.
// It defines all the routes and their handler functions.
// ─────────────────────────────────────────────────────────────────────────────

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};   // CorsLayer allows the browser to call this API
use tower_http::services::ServeDir;        // ServeDir serves the frontend UI files
use uuid::Uuid;                            // Uuid is used to identify records in the database

// Import our shared application state (config + db pool)
use crate::app::desktop::AppState;

// Import all the Data Transfer Objects (DTOs) — these are the shapes of
// incoming JSON request bodies for each operation
use crate::models::enrollment_payment::RecordEnrollmentPaymentDto;
use crate::models::guest::{CreateGuestDto, UpdateGuestDto};
use crate::models::room::{CreateRoomDto, UpdateRoomDto};
use crate::models::tenant::CreateTenantEnrollmentDto;
use crate::models::tenant_document::UploadTenantDocumentDto;
use crate::models::user::{AuthResponse, LoginDto, RegisterUserDto}; // ← FIX: Added AuthResponse import

// Import all service layers — these contain the actual business logic
use crate::services::{
    ledger_service::PayRentDto,
    AuthService, DocumentService, GuestService, LedgerService,
    ReceiptService, ReminderEngine, RoomService, TenantService,
};

// Import the response/model types we need to annotate our variables with
// This fixes the "never type fallback" errors — Rust needs to know the
// exact type being serialized by serde_json::to_value()
use crate::models::{
    EnrollmentPayment, Guest, Notification, Receipt, RentLedger,
    RoomResponse, TenantDocument, TenantEnrollmentDetail, TenantResponse,
};

// ─────────────────────────────────────────────────────────────────────────────
// ApiState wraps AppState in an Arc (Atomic Reference Count) so it can be
// safely shared across many async handler functions simultaneously
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Clone)]
pub struct ApiState {
    pub app: Arc<AppState>,
}

// ─────────────────────────────────────────────────────────────────────────────
// start_server: builds the router, binds the TCP port, and starts serving
// ─────────────────────────────────────────────────────────────────────────────
pub async fn start_server(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    // Wrap state so every handler can access it via State<ApiState>
    let api_state = ApiState { app: state.clone() };

    // Define all API routes and map them to handler functions
    let api_routes = Router::new()
        // ── Auth ──────────────────────────────────────────────────────────
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        // ── Rooms ─────────────────────────────────────────────────────────
        .route("/rooms", get(list_rooms).post(create_room))
        .route("/rooms/:id", put(update_room).delete(delete_room))
        // ── Guests (legacy support) ───────────────────────────────────────
        .route("/guests", get(list_guests).post(create_guest))
        .route("/guests/:id", get(get_guest).put(update_guest).delete(delete_guest))
        // ── Tenants & Enrollment ──────────────────────────────────────────
        .route("/tenants", get(list_tenants).post(enroll_tenant))
        .route("/tenants/:id", get(get_tenant))
        .route("/tenants/:id/payments", post(record_tenant_payment))
        .route("/tenants/:id/verify", post(verify_tenant))
        // ── Documents (proof attachments) ─────────────────────────────────
        .route("/documents", post(upload_document))
        .route("/documents/tenant/:id", get(list_tenant_documents))
        .route("/documents/:id", delete(delete_document))
        // ── Rent Ledger ───────────────────────────────────────────────────
        .route("/ledger", get(list_ledger))
        .route("/ledger/pay", post(pay_rent))
        .route("/ledger/tenant/:id", get(get_tenant_ledger))
        // ── Receipts ──────────────────────────────────────────────────────
        .route("/receipts", get(list_receipts))
        .route("/receipts/:id", get(get_receipt))
        .route("/receipts/tenant/:id", get(get_tenant_receipts))
        // ── Reminders & Notifications ─────────────────────────────────────
        .route("/reminders/run", post(run_reminders))
        .route("/notifications", get(list_notifications))
        .with_state(api_state); // attach the shared state to all routes

    // Combine API routes with static file serving for the frontend UI
    let app = Router::new()
        .nest("/api", api_routes)                  // all API endpoints under /api/
        .fallback_service(ServeDir::new("ui"))     // serve frontend from the /ui folder
        .layer(
            CorsLayer::new()
                .allow_origin(Any)     // allow requests from any origin (browser)
                .allow_methods(Any)    // allow GET, POST, PUT, DELETE etc.
                .allow_headers(Any),   // allow any headers (Content-Type, Authorization etc.)
        );

    // Bind to all network interfaces on the configured port
    let addr = format!("0.0.0.0:{}", state.config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("HTTP server listening on http://localhost:{}", state.config.app_port);

    // Start serving — this runs forever (until the process is killed)
    axum::serve(listener, app).await?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// AUTH HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// POST /api/auth/register — creates a new user account
async fn register(
    State(state): State<ApiState>,
    Json(mut dto): Json<RegisterUserDto>, // `mut` because we may modify username_or_login below
) -> Result<Json<serde_json::Value>, ApiError> {
    // If username is empty, fall back to using the email as the login identifier
    if dto.username_or_login.is_empty() {
        if let Some(email) = dto.email.take() {
            dto.username_or_login = email;
        }
    }
    // FIX: explicitly annotate the type so Rust knows what serde_json::to_value() is serializing
    let auth: AuthResponse = AuthService::register(
        &state.app.pool,
        dto,
        &state.app.config.jwt_secret,
        state.app.config.jwt_expires_in,
    )
    .await?;
    Ok(Json(serde_json::to_value(auth).unwrap()))
}

// POST /api/auth/login — authenticates a user and returns a JWT token
async fn login(
    State(state): State<ApiState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // FIX: explicitly annotate the type
    let auth: AuthResponse = AuthService::login(
        &state.app.pool,
        dto,
        &state.app.config.jwt_secret,
        state.app.config.jwt_expires_in,
    )
    .await?;
    Ok(Json(serde_json::to_value(auth).unwrap()))
}

// ─────────────────────────────────────────────────────────────────────────────
// ROOM HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// GET /api/rooms — returns all rooms
async fn list_rooms(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    // FIX: Vec<RoomResponse> tells Rust exactly what type the list contains
    let rooms: Vec<RoomResponse> = RoomService::list_rooms(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(rooms).unwrap()))
}

// POST /api/rooms — creates a new room
async fn create_room(
    State(state): State<ApiState>,
    Json(dto): Json<CreateRoomDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let room = RoomService::create_room(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(room).unwrap()))
}

// PUT /api/rooms/:id — updates an existing room by its UUID
async fn update_room(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateRoomDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Some if found and updated, None if not found
    let room = RoomService::update_room(&state.app.pool, id, dto).await?;
    Ok(Json(serde_json::json!({ "updated": room.is_some(), "room": room })))
}

// DELETE /api/rooms/:id — deletes a room by its UUID
async fn delete_room(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let success: bool = RoomService::delete_room(&state.app.pool, id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

// ─────────────────────────────────────────────────────────────────────────────
// GUEST HANDLERS (legacy support)
// ─────────────────────────────────────────────────────────────────────────────

// GET /api/guests
async fn list_guests(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let guests: Vec<Guest> = GuestService::list_guests(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(guests).unwrap()))
}

// POST /api/guests
async fn create_guest(
    State(state): State<ApiState>,
    Json(dto): Json<CreateGuestDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guest: Guest = GuestService::create_guest(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(guest).unwrap()))
}

// GET /api/guests/:id
async fn get_guest(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guest = GuestService::get_guest(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(guest).unwrap()))
}

// PUT /api/guests/:id
async fn update_guest(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateGuestDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let guest = GuestService::update_guest(&state.app.pool, id, dto).await?;
    Ok(Json(serde_json::to_value(guest).unwrap()))
}

// DELETE /api/guests/:id
async fn delete_guest(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let success: bool = GuestService::delete_guest(&state.app.pool, id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

// ─────────────────────────────────────────────────────────────────────────────
// TENANT HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// GET /api/tenants — lists all tenants
async fn list_tenants(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let tenants: Vec<TenantResponse> = TenantService::list_tenants(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(tenants).unwrap()))
}

// POST /api/tenants — enrolls a new tenant (check-in)
async fn enroll_tenant(
    State(state): State<ApiState>,
    Json(dto): Json<CreateTenantEnrollmentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let detail: TenantEnrollmentDetail =
        TenantService::create_enrollment(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}

// GET /api/tenants/:id — gets full details of one tenant
async fn get_tenant(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let detail: TenantEnrollmentDetail =
        TenantService::get_tenant_detail(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(detail).unwrap()))
}

// POST /api/tenants/:id/payments — records a payment for a tenant
async fn record_tenant_payment(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Json(mut dto): Json<RecordEnrollmentPaymentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Inject the tenant_id from the URL path into the DTO
    dto.tenant_id = id;
    let payment: EnrollmentPayment =
        TenantService::record_enrollment_payment(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(payment).unwrap()))
}

// POST /api/tenants/:id/verify — verifies and activates a tenant account
async fn verify_tenant(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // ActivationResult can be any serializable type your service returns
    let result = TenantService::verify_and_activate(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(result).unwrap()))
}

// ─────────────────────────────────────────────────────────────────────────────
// DOCUMENT HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// POST /api/documents — uploads a document for a tenant
async fn upload_document(
    State(state): State<ApiState>,
    Json(dto): Json<UploadTenantDocumentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let doc: TenantDocument = DocumentService::upload_document(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(doc).unwrap()))
}

// GET /api/documents/tenant/:id — lists all documents for a tenant
async fn list_tenant_documents(
    State(state): State<ApiState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let docs: Vec<TenantDocument> =
        DocumentService::list_tenant_documents(&state.app.pool, tenant_id).await?;
    Ok(Json(serde_json::to_value(docs).unwrap()))
}

// DELETE /api/documents/:id — deletes a document
async fn delete_document(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let success: bool = DocumentService::delete_document(&state.app.pool, id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

// ─────────────────────────────────────────────────────────────────────────────
// LEDGER HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// GET /api/ledger — lists all rent ledger entries
async fn list_ledger(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let ledger: Vec<RentLedger> = LedgerService::list_all_ledgers(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(ledger).unwrap()))
}

// POST /api/ledger/pay — records a rent payment
async fn pay_rent(
    State(state): State<ApiState>,
    Json(dto): Json<PayRentDto>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let res = LedgerService::pay_rent(&state.app.pool, dto).await?;
    Ok(Json(serde_json::to_value(res).unwrap()))
}

// GET /api/ledger/tenant/:id — gets ledger entries for a specific tenant
async fn get_tenant_ledger(
    State(state): State<ApiState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let ledger: Vec<RentLedger> =
        LedgerService::get_tenant_ledgers(&state.app.pool, tenant_id).await?;
    Ok(Json(serde_json::to_value(ledger).unwrap()))
}

// ─────────────────────────────────────────────────────────────────────────────
// RECEIPT HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// GET /api/receipts — lists all receipts
async fn list_receipts(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let receipts: Vec<Receipt> = ReceiptService::list_receipts(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(receipts).unwrap()))
}

// GET /api/receipts/:id — gets one receipt by ID
async fn get_receipt(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let receipt: Receipt = ReceiptService::get_receipt(&state.app.pool, id).await?;
    Ok(Json(serde_json::to_value(receipt).unwrap()))
}

// GET /api/receipts/tenant/:id — gets all receipts for a specific tenant
async fn get_tenant_receipts(
    State(state): State<ApiState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let receipts: Vec<Receipt> =
        ReceiptService::get_tenant_receipts(&state.app.pool, tenant_id).await?;
    Ok(Json(serde_json::to_value(receipts).unwrap()))
}

// ─────────────────────────────────────────────────────────────────────────────
// REMINDER & NOTIFICATION HANDLERS
// ─────────────────────────────────────────────────────────────────────────────

// POST /api/reminders/run — manually triggers the reminder cycle
async fn run_reminders(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, ApiError> {
    let summary = ReminderEngine::run_reminder_cycle(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(summary).unwrap()))
}

// GET /api/notifications — lists all pending notifications
async fn list_notifications(
    State(state): State<ApiState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let notifications: Vec<Notification> =
        ReminderEngine::list_notifications(&state.app.pool).await?;
    Ok(Json(serde_json::to_value(notifications).unwrap()))
}

// ─────────────────────────────────────────────────────────────────────────────
// ApiError — a custom error type for all handler functions
//
// Why do we need this?
// Axum handlers must return something that implements IntoResponse.
// sqlx::Error and other errors don't implement IntoResponse by default,
// so we wrap them in our own ApiError which knows how to become an HTTP response.
// ─────────────────────────────────────────────────────────────────────────────
pub struct ApiError(String);

// Allow converting a plain String into an ApiError
impl From<String> for ApiError {
    fn from(value: String) -> Self {
        ApiError(value)
    }
}

// FIX: Allow converting sqlx::Error into ApiError automatically.
// This is what makes the `?` operator work in handler functions —
// when a database query fails, Rust automatically calls this to wrap the error.
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError(e.to_string()) // convert the DB error to a readable string message
    }
}

// Tell Axum how to turn an ApiError into an HTTP response.
// We return HTTP 400 Bad Request with a JSON body: { "error": "..." }
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": self.0 })),
        )
            .into_response()
    }
}