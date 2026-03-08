use anyhow::{Context, Result, anyhow, bail};
use config::{Config, File, FileFormat};
use serde::Deserialize;
use std::collections::BTreeMap;

const DEFAULT_SETTINGS_TOML: &str = r#"
[app]
host = "0.0.0.0"
port = 8080
auto_migrate = true
allowed_origins = []

[database]
url = "postgres://ordering_food:ordering_food@127.0.0.1:5432/ordering_food"
max_connections = 10

[redis]
url = "redis://127.0.0.1:6379"
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub app: AppSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
    pub auto_migrate: bool,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub url: String,
}

impl Settings {
    pub fn from_env() -> Result<Self> {
        let overrides = std::env::vars().filter(|(key, _)| is_supported_env_key(key));
        Self::from_overrides(overrides)
    }

    pub fn from_overrides<I, K, V>(overrides: I) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let overrides = overrides
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect::<BTreeMap<String, String>>();

        let override_toml = build_override_toml(&overrides)?;

        let mut builder =
            Config::builder().add_source(File::from_str(DEFAULT_SETTINGS_TOML, FileFormat::Toml));

        if !override_toml.is_empty() {
            builder = builder.add_source(File::from_str(&override_toml, FileFormat::Toml));
        }

        builder
            .build()
            .context("failed to build configuration")?
            .try_deserialize::<Settings>()
            .context("failed to deserialize configuration")
    }
}

impl AppSettings {
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn build_override_toml(overrides: &BTreeMap<String, String>) -> Result<String> {
    let mut lines = Vec::new();

    if let Some(host) = overrides.get("APP__HOST") {
        lines.push(format!("app.host = {}", quote_toml_string(host)));
    }

    if let Some(port) = overrides.get("APP__PORT") {
        let port = port
            .parse::<u16>()
            .map_err(|_| anyhow!("APP__PORT must be a valid u16, got `{port}`"))?;
        lines.push(format!("app.port = {port}"));
    }

    if let Some(auto_migrate) = overrides.get("APP__AUTO_MIGRATE") {
        let auto_migrate = parse_bool("APP__AUTO_MIGRATE", auto_migrate)?;
        lines.push(format!("app.auto_migrate = {auto_migrate}"));
    }

    if let Some(origins) = overrides.get("APP__ALLOWED_ORIGINS") {
        let origins = origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(quote_toml_string)
            .collect::<Vec<_>>();

        lines.push(format!("app.allowed_origins = [{}]", origins.join(", ")));
    }

    if let Some(database_url) = overrides.get("DATABASE__URL") {
        lines.push(format!(
            "database.url = {}",
            quote_toml_string(database_url)
        ));
    }

    if let Some(max_connections) = overrides.get("DATABASE__MAX_CONNECTIONS") {
        let max_connections = max_connections.parse::<u32>().map_err(|_| {
            anyhow!("DATABASE__MAX_CONNECTIONS must be a valid u32, got `{max_connections}`")
        })?;

        lines.push(format!("database.max_connections = {max_connections}"));
    }

    if let Some(redis_url) = overrides.get("REDIS__URL") {
        lines.push(format!("redis.url = {}", quote_toml_string(redis_url)));
    }

    Ok(lines.join("\n"))
}

fn parse_bool(key: &str, value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => bail!("{key} must be a valid boolean, got `{value}`"),
    }
}

fn quote_toml_string(value: &str) -> String {
    serde_json::to_string(value).expect("failed to serialize string")
}

fn is_supported_env_key(key: &str) -> bool {
    matches!(
        key,
        "APP__HOST"
            | "APP__PORT"
            | "APP__AUTO_MIGRATE"
            | "APP__ALLOWED_ORIGINS"
            | "DATABASE__URL"
            | "DATABASE__MAX_CONNECTIONS"
            | "REDIS__URL"
    )
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn loads_defaults_when_no_overrides_exist() {
        let settings = Settings::from_overrides(std::iter::empty::<(String, String)>()).unwrap();

        assert_eq!(settings.app.host, "0.0.0.0");
        assert_eq!(settings.app.port, 8080);
        assert!(settings.app.auto_migrate);
        assert!(settings.app.allowed_origins.is_empty());
        assert_eq!(
            settings.database.url,
            "postgres://ordering_food:ordering_food@127.0.0.1:5432/ordering_food"
        );
        assert_eq!(settings.database.max_connections, 10);
        assert_eq!(settings.redis.url, "redis://127.0.0.1:6379");
    }

    #[test]
    fn applies_env_style_overrides() {
        let settings = Settings::from_overrides([
            ("APP__PORT", "9090"),
            ("APP__AUTO_MIGRATE", "false"),
            (
                "APP__ALLOWED_ORIGINS",
                "http://localhost:3000, http://127.0.0.1:5173",
            ),
            (
                "DATABASE__URL",
                "postgres://override:override@127.0.0.1:5432/override_db",
            ),
            ("DATABASE__MAX_CONNECTIONS", "25"),
            ("REDIS__URL", "redis://127.0.0.1:6380"),
        ])
        .unwrap();

        assert_eq!(settings.app.port, 9090);
        assert!(!settings.app.auto_migrate);
        assert_eq!(
            settings.app.allowed_origins,
            vec![
                "http://localhost:3000".to_string(),
                "http://127.0.0.1:5173".to_string(),
            ]
        );
        assert_eq!(
            settings.database.url,
            "postgres://override:override@127.0.0.1:5432/override_db"
        );
        assert_eq!(settings.database.max_connections, 25);
        assert_eq!(settings.redis.url, "redis://127.0.0.1:6380");
    }

    #[test]
    fn rejects_invalid_numeric_overrides() {
        let error = Settings::from_overrides([("APP__PORT", "not-a-port")]).unwrap_err();

        assert!(error.to_string().contains("APP__PORT"));
    }
}
