use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
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
use crate::models::user::{LoginDto, RegisterUserDto};
use crate::services::{AuthService, GuestService, RoomService, TenantService};

#[derive(Clone)]
pub struct ApiState {
    pub app: Arc<AppState>,
}

pub async fn start_server(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let api_state = ApiState { app: state.clone() };

    let api_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/rooms", get(list_rooms).post(create_room))
        .route("/rooms/:id", put(update_room).delete(delete_room))
        .route("/guests", get(list_guests).post(create_guest))
        .route("/guests/:id", get(get_guest).put(update_guest).delete(delete_guest))
        .route("/tenants", get(list_tenants).post(enroll_tenant))
        .route("/tenants/:id", get(get_tenant))
        .route("/tenants/:id/payments", post(record_tenant_payment))
        .route("/tenants/:id/verify", post(verify_tenant))
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
