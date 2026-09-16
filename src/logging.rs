use std::fs::File;
use std::path::Path;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub fn init(level: Level, log_file: Option<&Path>) {
    let filter = EnvFilter::new(level.as_str());
    let subscriber = tracing_subscriber::registry().with(filter);

    if let Some(path) = log_file {
        match open_log_file(path) {
            Ok(file) => {
                subscriber
                    .with(fmt::layer().with_writer(file).with_ansi(false))
                    .with(fmt::layer().with_writer(std::io::stderr))
                    .init();
                return;
            }
            Err(e) => eprintln!("Failed to open log file {}: {e}", path.display()),
        }
    }

    subscriber
        .with(fmt::layer().with_writer(std::io::stderr))
        .init();
}

/// Open the log file for appending, creating it if it is missing.
fn open_log_file(path: &Path) -> std::io::Result<File> {
    File::options().create(true).append(true).open(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_reach_the_log_file() {
        let path = std::env::temp_dir().join(format!("gh-tray-events-{}.log", std::process::id()));
        std::fs::remove_file(&path).ok();

        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("info"))
            .with(
                fmt::layer()
                    .with_writer(open_log_file(&path).unwrap())
                    .with_ansi(false),
            );
        {
            let _guard = tracing::subscriber::set_default(subscriber);
            tracing::info!("event from the test");
        }

        let contents = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert!(
            contents.contains("event from the test"),
            "log held: {contents:?}"
        );
    }

    #[test]
    fn open_log_file_appends_to_an_existing_file() {
        let path =
            std::env::temp_dir().join(format!("gh-tray-log-test-{}.log", std::process::id()));
        std::fs::write(&path, "existing\n").unwrap();

        {
            use std::io::Write;
            let mut file = open_log_file(&path).expect("open failed");
            file.write_all(b"appended\n").unwrap();
        }

        let contents = std::fs::read_to_string(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(contents, "existing\nappended\n");
    }

    #[test]
    fn open_log_file_creates_a_missing_file() {
        let path = std::env::temp_dir().join(format!("gh-tray-new-{}.log", std::process::id()));
        std::fs::remove_file(&path).ok();

        let created = open_log_file(&path).is_ok();
        let exists = path.exists();
        std::fs::remove_file(&path).ok();

        assert!(created && exists);
    }
}
