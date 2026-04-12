use anyhow::{Context, Result};
use ordering_food_app_support::{
    config::Settings,
    observability::{format_anyhow_chain, init_tracing, install_panic_hook},
    runtime::SystemClock,
};
use ordering_food_bootstrap::run_default_data_bootstrap;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::error;

#[tokio::main]
async fn main() -> Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(error) if error.not_found() => {}
        Err(error) => {
            eprintln!("failed to load .env: {error}");
            return Err(error.into());
        }
    }

    init_tracing();
    install_panic_hook();

    run().await.map_err(|error| {
        error!(
            error = %error,
            error_chain = %format_anyhow_chain(&error),
            "bootstrap command terminated with error"
        );
        error
    })
}

async fn run() -> Result<()> {
    let settings = Settings::from_env()?;
    let pg_pool = PgPoolOptions::new()
        .max_connections(settings.database.max_connections)
        .connect(&settings.database.url)
        .await
        .context("failed to connect to postgres")?;

    run_default_data_bootstrap(&settings, pg_pool, Arc::new(SystemClock)).await
}
