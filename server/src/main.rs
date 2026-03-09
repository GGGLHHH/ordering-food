use ordering_food_server::app::run;
use ordering_food_shared::observability::{format_anyhow_chain, init_tracing, install_panic_hook};
use tracing::error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        error!(error = %error, error_chain = %format_anyhow_chain(&error), "application terminated with error");
        error
    })
}
