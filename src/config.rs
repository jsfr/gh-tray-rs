use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::Level;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub account: Option<String>,
    pub poll_interval: Duration,
    pub log_level: Level,
    pub hotkey: String,
    pub log_file: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            account: None,
            poll_interval: Duration::from_secs(120),
            log_level: Level::INFO,
            hotkey: "Ctrl+Alt+Shift+G".to_string(),
            log_file: None,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ConfigFile {
    account: Option<String>,
    poll_interval: Option<u64>,
    log_level: Option<String>,
    hotkey: Option<String>,
    log_file: Option<String>,
}

fn parse_log_level(s: &str) -> Option<Level> {
    match s.to_lowercase().as_str() {
        "trace" => Some(Level::TRACE),
        "debug" => Some(Level::DEBUG),
        "information" | "info" => Some(Level::INFO),
        "warning" | "warn" => Some(Level::WARN),
        "error" => Some(Level::ERROR),
        _ => None,
    }
}

impl From<ConfigFile> for AppConfig {
    fn from(cf: ConfigFile) -> Self {
        let defaults = AppConfig::default();
        AppConfig {
            account: cf.account,
            poll_interval: cf
                .poll_interval
                .map(Duration::from_secs)
                .unwrap_or(defaults.poll_interval),
            log_level: cf
                .log_level
                .as_deref()
                .and_then(parse_log_level)
                .unwrap_or(defaults.log_level),
            hotkey: cf.hotkey.unwrap_or(defaults.hotkey),
            log_file: cf.log_file.map(PathBuf::from),
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("gh-tray").join("config.json")
}

pub fn load() -> AppConfig {
    let path = config_path();

    if !path.exists() {
        return AppConfig::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(json) => match serde_json::from_str::<ConfigFile>(&json) {
            Ok(cf) => cf.into(),
            Err(e) => {
                eprintln!("Failed to parse config file {}: {e}", path.display());
                AppConfig::default()
            }
        },
        Err(e) => {
            eprintln!("Failed to read config file {}: {e}", path.display());
            AppConfig::default()
        }
    }
}

/// Apply environment variable overrides to config.
pub fn apply_env_overrides(config: &mut AppConfig) {
    if let Ok(v) = std::env::var("GH_TRAY_POLL_INTERVAL") {
        match v.parse::<u64>() {
            Ok(secs) => config.poll_interval = Duration::from_secs(secs),
            Err(_) => eprintln!(
                "Invalid GH_TRAY_POLL_INTERVAL: {v}, using default {}s",
                config.poll_interval.as_secs()
            ),
        }
    }

    if let Ok(v) = std::env::var("GH_TRAY_LOG_LEVEL") {
        match parse_log_level(&v) {
            Some(level) => config.log_level = level,
            None => eprintln!("Invalid GH_TRAY_LOG_LEVEL: {v}, using default"),
        }
    }

    if let Ok(v) = std::env::var("GH_TRAY_HOTKEY") {
        config.hotkey = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let c = AppConfig::default();
        assert_eq!(c.poll_interval, Duration::from_secs(120));
        assert_eq!(c.log_level, Level::INFO);
        assert_eq!(c.hotkey, "Ctrl+Alt+Shift+G");
        assert!(c.account.is_none());
        assert!(c.log_file.is_none());
    }

    #[test]
    fn parse_full_config_json() {
        let json = r#"{
            "account": "myuser",
            "pollInterval": 60,
            "logLevel": "Debug",
            "hotkey": "Ctrl+G",
            "logFile": "/tmp/gh-tray.log"
        }"#;
        let cf: ConfigFile = serde_json::from_str(json).unwrap();
        let config: AppConfig = cf.into();
        assert_eq!(config.account.as_deref(), Some("myuser"));
        assert_eq!(config.poll_interval, Duration::from_secs(60));
        assert_eq!(config.log_level, Level::DEBUG);
        assert_eq!(config.hotkey, "Ctrl+G");
        assert_eq!(
            config.log_file.unwrap().to_str().unwrap(),
            "/tmp/gh-tray.log"
        );
    }

    #[test]
    fn parse_partial_config_json() {
        let json = r#"{ "pollInterval": 30 }"#;
        let cf: ConfigFile = serde_json::from_str(json).unwrap();
        let config: AppConfig = cf.into();
        assert_eq!(config.poll_interval, Duration::from_secs(30));
        assert_eq!(config.log_level, Level::INFO); // default
        assert_eq!(config.hotkey, "Ctrl+Alt+Shift+G"); // default
    }

    #[test]
    fn parse_empty_config_json() {
        let json = "{}";
        let cf: ConfigFile = serde_json::from_str(json).unwrap();
        let config: AppConfig = cf.into();
        assert_eq!(config.poll_interval, Duration::from_secs(120));
    }

    #[test]
    fn parse_log_level_variants() {
        assert_eq!(parse_log_level("Information"), Some(Level::INFO));
        assert_eq!(parse_log_level("info"), Some(Level::INFO));
        assert_eq!(parse_log_level("Debug"), Some(Level::DEBUG));
        assert_eq!(parse_log_level("Warning"), Some(Level::WARN));
        assert_eq!(parse_log_level("Error"), Some(Level::ERROR));
        assert_eq!(parse_log_level("Trace"), Some(Level::TRACE));
        assert_eq!(parse_log_level("invalid"), None);
    }
}
