mod app;
mod config;
mod db;
mod models;
mod repository;
mod services;

use config::Config;
use db::postgres::create_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing / logger subscriber
    tracing_subscriber::fmt::init();

    // Load environment configuration
    let config = Config::from_env();
    tracing::info!("Initializing PG CRM Desktop App with database: {}", config.database_url);

    // Initialize database pool
    let pool = match create_pool(&config.database_url).await {
        Ok(pool) => {
            println!("Successfully connected to PostgreSQL database at {}", config.database_url);
            pool
        }
        Err(err) => {
            eprintln!("Failed to connect to PostgreSQL database: {}", err);
            eprintln!("Please ensure PostgreSQL is running and credentials in .env are correct.");
            eprintln!("Default DATABASE_URL: postgres://postgres:postgres@localhost:5432/pg_crm_db");
            return Err(Box::new(err));
        }
    };

    // Run Desktop App
    app::run_desktop_app(config, pool)
    .await
    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(())
}
