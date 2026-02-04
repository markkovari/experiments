use axum_test_it_is_whay_it_is::{create_app, AppConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,axum_test_it_is_whay_it_is=debug".into()),
        )
        .init();

    // Load configuration
    let config = AppConfig::from_env()?;

    tracing::info!("Starting server on {}", config.server_addr);

    // Create and run the app
    create_app(config).await?;

    Ok(())
}
