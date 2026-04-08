use std::path::Path;
use tracing::Level;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

pub fn init(level: Level, log_file: Option<&Path>) {
    let filter = EnvFilter::new(level.as_str());

    let subscriber = tracing_subscriber::registry().with(filter);

    if let Some(path) = log_file {
        let dir = path.parent().unwrap_or(Path::new("."));
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_appender = tracing_appender::rolling::never(dir, filename);
        let file_layer = fmt::layer().with_writer(file_appender).with_ansi(false);
        let stderr_layer = fmt::layer().with_writer(std::io::stderr);

        subscriber.with(file_layer).with(stderr_layer).init();
    } else {
        let stderr_layer = fmt::layer().with_writer(std::io::stderr);
        subscriber.with(stderr_layer).init();
    }
}
