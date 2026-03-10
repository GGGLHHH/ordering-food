use anyhow::Error;
use std::{error::Error as StdError, panic::PanicHookInfo};
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("ordering_food_api=debug,ordering_food_server=debug,tower_http=info")
    });

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
