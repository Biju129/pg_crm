use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Owner,
    Admin,
    Tenant,
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Owner => "OWNER".to_string(),
            UserRole::Admin => "ADMIN".to_string(),
            UserRole::Tenant => "TENANT".to_string(),
        }
    }
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.as_str() {
            "OWNER" => UserRole::Owner,
            "ADMIN" => UserRole::Admin,
            _ => UserRole::Tenant,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username_or_login: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub tenant_id: Option<Uuid>,
    pub is_active: bool,
    pub must_change_password: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username_or_login: String,
    pub role: String,
    pub tenant_id: Option<Uuid>,
    pub is_active: bool,
    pub must_change_password: bool,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username_or_login: user.username_or_login,
            role: user.role,
            tenant_id: user.tenant_id,
            is_active: user.is_active,
            must_change_password: user.must_change_password,
            last_login_at: user.last_login_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterUserDto {
    #[serde(default)]
    pub username_or_login: String,
    pub email: Option<String>,
    pub password: String,
    pub role: Option<String>,
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginDto {
    pub username_or_login: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}
