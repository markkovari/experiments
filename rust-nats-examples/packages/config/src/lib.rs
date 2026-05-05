use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub nats: NatsConfig,
    pub mongodb: Option<MongoConfig>,
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NatsConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MongoConfig {
    pub url: String,
    pub db_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(File::with_name("config/local").required(false))
            // The library merge logic often fails with mixed case env vars.
            // We build the base config from files first.
            .build()?;

        let mut config: AppConfig = s.try_deserialize()?;
        
        // 🚀 ROBUST K8S OVERRIDES
        // Instead of relying on the library to merge, we manually inject 
        // the standard K8s environment variables if they exist.
        
        if let Ok(url) = std::env::var("APP_NATS__URL") {
            config.nats.url = url;
        }

        if let Ok(url) = std::env::var("APP_MONGODB__URL") {
            if let Some(ref mut mongo) = config.mongodb {
                mongo.url = url;
            } else {
                // If the section didn't exist in TOML, initialize it
                config.mongodb = Some(MongoConfig {
                    url: url.clone(),
                    db_name: std::env::var("APP_MONGODB__DB_NAME").unwrap_or_else(|_| "default_db".into()),
                });
            }
        }
        
        // Explicitly check for db_name override
        if let Ok(db_name) = std::env::var("APP_MONGODB__DB_NAME") {
            if let Some(ref mut mongo) = config.mongodb {
                mongo.db_name = db_name;
            }
        }

        if let Ok(secret) = std::env::var("APP_AUTH__JWT_SECRET") {
            if let Some(ref mut auth) = config.auth {
                auth.jwt_secret = secret;
            } else {
                config.auth = Some(AuthConfig { jwt_secret: secret });
            }
        }

        println!("🚀 CONFIG READY | NATS: {} | MONGO_DB: {}", 
            config.nats.url, 
            config.mongodb.as_ref().map(|m| m.db_name.as_str()).unwrap_or("none")
        );
        
        Ok(config)
    }
}
