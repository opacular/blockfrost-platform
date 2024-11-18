use crate::cli::Config;
use tracing_subscriber::fmt::format::Format;

/// Sets up the tracing subscriber with the provided configuration.
pub fn setup_tracing(config: &Config) {
    tracing_subscriber::fmt()
        .with_max_level(config.log_level)
        .event_format(
            Format::default()
                .with_ansi(true)
                .with_level(true)
                .with_target(false)
                .compact(),
        )
        .init();
}
