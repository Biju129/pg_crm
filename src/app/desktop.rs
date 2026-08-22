use crate::config::Config;
// use crate::db::run_migrations;
use crate::db::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
}

impl AppState {
    pub fn new(pool: DbPool, config: Config) -> Self {
        Self { pool, config }
    }
}

pub async fn run_desktop_app(config: Config, pool: DbPool) -> Result<(), Box<dyn std::error::Error>> {
    // run_migrations(&pool).await?;

    let state = Arc::new(AppState::new(pool, config.clone()));

    println!("==========================================================");
    println!(" PG CRM Desktop Application (Rust + PostgreSQL Database)");
    println!(" Backend logic and desktop state initialized successfully!");
    println!(" App Port: {}", config.app_port);
    println!(" Open UI: http://localhost:{}", config.app_port);
    println!(" Tenant API: POST /api/tenants (enroll)");
    println!("            POST /api/tenants/:id/payments");
    println!("            POST /api/tenants/:id/verify");
    println!("==========================================================");

    super::server::start_server(state).await
}