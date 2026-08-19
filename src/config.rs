use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub app_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/pg_crm_db".to_string());

        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "super_secret_pg_crm_key_change_in_production".to_string());

        let jwt_expires_in = env::var("JWT_EXPIRES_IN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(604800);

        let app_port = env::var("APP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        Self {
            database_url,
            jwt_secret,
            jwt_expires_in,
            app_port,
        }
    }
}
