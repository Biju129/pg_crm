use crate::db::DbPool;
use crate::models::user::{AuthResponse, LoginDto, RegisterUserDto, UserResponse};
use crate::repository::UserRepository;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub struct AuthService;

impl AuthService {
    pub async fn register(
        pool: &DbPool,
        mut dto: RegisterUserDto,
        jwt_secret: &str,
        expires_in: u64,
    ) -> Result<AuthResponse, String> {
        if dto.username_or_login.trim().is_empty() {
            dto.username_or_login = dto
                .email
                .take()
                .filter(|e| !e.trim().is_empty())
                .ok_or_else(|| "Username or email is required".to_string())?;
        }

        let existing = UserRepository::find_by_username(pool, &dto.username_or_login)
            .await
            .map_err(|e| e.to_string())?;

        if existing.is_some() {
            return Err("User already exists with this login username".to_string());
        }

        // Hash password using Argon2
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(dto.password.as_bytes(), &salt)
            .map_err(|e| e.to_string())?
            .to_string();

        let role = dto.role.unwrap_or_else(|| "TENANT".to_string());

        let user = UserRepository::create(
            pool,
            &dto.username_or_login,
            &password_hash,
            &role,
            dto.tenant_id,
            true,
        )
        .await
        .map_err(|e| e.to_string())?;

        let token = Self::generate_token(&user.id.to_string(), &user.role, jwt_secret, expires_in)?;

        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    pub async fn login(
        pool: &DbPool,
        dto: LoginDto,
        jwt_secret: &str,
        expires_in: u64,
    ) -> Result<AuthResponse, String> {
        let user = UserRepository::find_by_username(pool, &dto.username_or_login)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Invalid credentials".to_string())?;

        if !user.is_active {
            return Err("Account is inactive".to_string());
        }

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| e.to_string())?;
        Argon2::default()
            .verify_password(dto.password.as_bytes(), &parsed_hash)
            .map_err(|_| "Invalid credentials".to_string())?;

        let token = Self::generate_token(&user.id.to_string(), &user.role, jwt_secret, expires_in)?;

        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    pub fn generate_token(
        user_id: &str,
        role: &str,
        secret: &str,
        expires_in: u64,
    ) -> Result<String, String> {
        let now = chrono::Utc::now().timestamp() as u64;
        let claims = Claims {
            sub: user_id.to_string(),
            role: role.to_string(),
            exp: (now + expires_in) as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .map_err(|e| e.to_string())
    }
}
