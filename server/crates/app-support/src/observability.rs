use anyhow::Error;
use std::{error::Error as StdError, panic::PanicHookInfo};
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

const APP_LOG_LEVEL_ENV_VAR: &str = "APP__LOG_LEVEL";
const DEFAULT_LOG_DIRECTIVE: &str =
    "ordering_food_api=debug,ordering_food_server=debug,tower_http=info";

pub fn init_tracing() {
    let env_filter = build_env_filter(
        std::env::var("RUST_LOG").ok(),
        std::env::var(APP_LOG_LEVEL_ENV_VAR).ok(),
    );

    tracing_subscriber::registry()
        .with(env_filter)
        .with(ErrorLayer::default())
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_target(false),
        )
        .init();
}

fn build_env_filter(rust_log: Option<String>, app_log_level: Option<String>) -> EnvFilter {
    let directive = resolve_log_directive(rust_log, app_log_level);
    EnvFilter::try_new(directive).expect("log directive should be valid")
}

fn resolve_log_directive(rust_log: Option<String>, app_log_level: Option<String>) -> String {
    if let Some(rust_log) = rust_log
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if EnvFilter::try_new(rust_log).is_ok() {
            return rust_log.to_string();
        }

        eprintln!("invalid RUST_LOG value `{rust_log}`, falling back to {APP_LOG_LEVEL_ENV_VAR}");
    }

    app_log_level_directive(app_log_level).unwrap_or_else(|| DEFAULT_LOG_DIRECTIVE.to_string())
}

fn app_log_level_directive(app_log_level: Option<String>) -> Option<String> {
    let raw_level = app_log_level?;
    let normalized_level = raw_level.trim().to_ascii_lowercase();

    if normalized_level.is_empty() {
        return None;
    }

    if !is_supported_log_level(&normalized_level) {
        eprintln!(
            "invalid {APP_LOG_LEVEL_ENV_VAR} value `{raw_level}`, falling back to default log filter"
        );
        return None;
    }

    Some(format!(
        "ordering_food_api={0},ordering_food_server={0},tower_http={0}",
        normalized_level
    ))
}

fn is_supported_log_level(level: &str) -> bool {
    matches!(level, "trace" | "debug" | "info" | "warn" | "error")
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_payload(panic_info);
        let location = panic_info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown".to_string());

        error!(panic.payload = %payload, panic.location = %location, "panic occurred");
    }));
}

pub fn format_anyhow_chain(error: &Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}

pub fn format_error_chain(error: &(dyn StdError + Send + Sync + 'static)) -> String {
    let mut chain = vec![error.to_string()];
    let mut current = error.source();

    while let Some(source) = current {
        chain.push(source.to_string());
        current = source.source();
    }

    chain.join(": ")
}

fn panic_payload(panic_info: &PanicHookInfo<'_>) -> String {
    if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
        return (*message).to_string();
    }

    if let Some(message) = panic_info.payload().downcast_ref::<String>() {
        return message.clone();
    }

    "Box<Any>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_log_level_maps_to_scoped_directive() {
        let directive = resolve_log_directive(None, Some("error".to_string()));

        assert_eq!(
            directive,
            "ordering_food_api=error,ordering_food_server=error,tower_http=error"
        );
    }

    #[test]
    fn rust_log_takes_precedence_over_app_log_level() {
        let directive = resolve_log_directive(
            Some("warn,tower_http=error".to_string()),
            Some("error".to_string()),
        );

        assert_eq!(directive, "warn,tower_http=error");
    }

    #[test]
    fn invalid_rust_log_falls_back_to_app_log_level() {
        let directive = resolve_log_directive(
            Some("ordering_food_api==warn".to_string()),
            Some("warn".to_string()),
        );

        assert_eq!(
            directive,
            "ordering_food_api=warn,ordering_food_server=warn,tower_http=warn"
        );
    }

    #[test]
    fn invalid_app_log_level_falls_back_to_default_directive() {
        let directive = resolve_log_directive(None, Some("verbose".to_string()));

        assert_eq!(directive, DEFAULT_LOG_DIRECTIVE);
    }
}
