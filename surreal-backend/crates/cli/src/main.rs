use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use surreal_api::{create_router, AppState};
use surreal_db::Database;
use surreal_migrations::MigrationRunner;

#[derive(Parser)]
#[command(name = "surreal-backend")]
#[command(about = "Veterinary Clinic Backend with SurrealDB", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    Serve {
        /// Server port (overrides PORT env var)
        #[arg(short, long)]
        port: Option<u16>,

        /// Run migrations on startup
        #[arg(short, long, default_value = "true")]
        migrate: bool,

        /// Seed database with sample data
        #[arg(short, long, default_value = "false")]
        seed: bool,
    },
    /// Run database migrations
    Migrate {
        /// Seed database with sample data
        #[arg(short, long, default_value = "false")]
        seed: bool,
    },
}

async fn connect_database() -> Result<Database> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "rocksdb://./data/surrealdb".to_string());

    let db = if database_url.starts_with("http://")
        || database_url.starts_with("https://")
        || database_url.starts_with("ws://")
        || database_url.starts_with("wss://")
    {
        // Remote connection
        let username = env::var("DATABASE_USERNAME").unwrap_or_else(|_| "root".to_string());
        let password = env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "root".to_string());
        info!("Connecting to remote database: {}", database_url);
        Database::connect_remote(&database_url, &username, &password).await?
    } else if database_url.starts_with("mem://") {
        // In-memory
        info!("Using in-memory database");
        Database::new_in_memory().await?
    } else {
        // Local RocksDB
        let path = database_url
            .strip_prefix("rocksdb://")
            .unwrap_or(&database_url);
        info!("Using local RocksDB database: {}", path);
        Database::new(path).await?
    };

    Ok(db)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve {
            port,
            migrate,
            seed,
        } => {
            info!("Starting Veterinary Clinic API Server");

            // Initialize database using DATABASE_URL env var
            let db = connect_database().await?;

            // Run migrations if requested
            if migrate {
                info!("Running migrations");
                let runner = MigrationRunner::new(db.clone());
                if seed {
                    runner.run_with_seed().await?;
                } else {
                    runner.run().await?;
                }
            }

            // Create app state
            let state = AppState::new(db);

            // Create router
            let app = create_router(state);

            // Get port from CLI arg, then PORT env var, then default to 3000
            let server_port = port
                .or_else(|| env::var("PORT").ok().and_then(|p| p.parse().ok()))
                .unwrap_or(3000);

            // Start server
            let addr = format!("0.0.0.0:{}", server_port);
            info!("Server listening on {}", addr);

            let listener = tokio::net::TcpListener::bind(&addr).await?;
            axum::serve(listener, app).await?;

            Ok(())
        }
        Commands::Migrate { seed } => {
            info!("Running database migrations");

            let db = connect_database().await?;
            let runner = MigrationRunner::new(db);

            if seed {
                runner.run_with_seed().await?;
            } else {
                runner.run().await?;
            }

            info!("Migrations completed successfully");
            Ok(())
        }
    }
}
