use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub machine_id: String,
    pub lantern_dir: PathBuf,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub run_dir: PathBuf,
    pub database_url: String,
    pub temporal_address: String,
    pub temporal_namespace: String,
    pub relay_socket_path: PathBuf,
    pub relay_pid_path: PathBuf,
    pub reconciliation_interval_secs: u64,
    pub ack_timeout_secs: u64,
    pub ack_retry_interval_secs: u64,
    pub stale_threshold_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().expect("home directory required");
        let lantern_dir = home.join(".lantern");
        let data_dir = lantern_dir.join("data");
        let relay_data = data_dir.join("relay");
        let run_dir = lantern_dir.join("run");

        Self {
            machine_id: hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            lantern_dir: lantern_dir.clone(),
            data_dir: data_dir.clone(),
            config_dir: lantern_dir.join("config"),
            logs_dir: lantern_dir.join("logs"),
            run_dir: run_dir.clone(),
            database_url: format!(
                "sqlite://{}",
                relay_data.join("lantern.db").to_string_lossy()
            ),
            temporal_address: "127.0.0.1:8243".to_string(),
            temporal_namespace: "default".to_string(),
            relay_socket_path: run_dir.join("relay.sock"),
            relay_pid_path: run_dir.join("relay.pid"),
            reconciliation_interval_secs: 5,
            ack_timeout_secs: 30,
            ack_retry_interval_secs: 30,
            stale_threshold_secs: 300,
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = dirs::home_dir()
            .expect("home directory required")
            .join(".lantern")
            .join("config")
            .join("lantern.toml");

        let defaults = Config::default();
        let mut config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            defaults.clone()
        };

        // Merge any empty / zero fields back to defaults so partial configs work
        if config.machine_id.is_empty() {
            config.machine_id = defaults.machine_id;
        }
        if config.lantern_dir.as_os_str().is_empty() {
            config.lantern_dir = defaults.lantern_dir;
        }
        if config.data_dir.as_os_str().is_empty() {
            config.data_dir = defaults.data_dir;
        }
        if config.config_dir.as_os_str().is_empty() {
            config.config_dir = defaults.config_dir;
        }
        if config.logs_dir.as_os_str().is_empty() {
            config.logs_dir = defaults.logs_dir;
        }
        if config.run_dir.as_os_str().is_empty() {
            config.run_dir = defaults.run_dir;
        }
        if config.database_url.is_empty() {
            config.database_url = defaults.database_url;
        }
        if config.temporal_address.is_empty() {
            config.temporal_address = defaults.temporal_address;
        }
        if config.temporal_namespace.is_empty() {
            config.temporal_namespace = defaults.temporal_namespace;
        }
        if config.relay_socket_path.as_os_str().is_empty() {
            config.relay_socket_path = defaults.relay_socket_path;
        }
        if config.relay_pid_path.as_os_str().is_empty() {
            config.relay_pid_path = defaults.relay_pid_path;
        }
        if config.reconciliation_interval_secs == 0 {
            config.reconciliation_interval_secs = defaults.reconciliation_interval_secs;
        }
        if config.ack_timeout_secs == 0 {
            config.ack_timeout_secs = defaults.ack_timeout_secs;
        }
        if config.ack_retry_interval_secs == 0 {
            config.ack_retry_interval_secs = defaults.ack_retry_interval_secs;
        }
        if config.stale_threshold_secs == 0 {
            config.stale_threshold_secs = defaults.stale_threshold_secs;
        }

        config.ensure_dirs()?;
        Ok(config)
    }

    fn ensure_dirs(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.lantern_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(self.data_dir.join("relay"))?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.logs_dir)?;
        std::fs::create_dir_all(&self.run_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.machine_id.is_empty());
        assert_eq!(config.temporal_address, "127.0.0.1:8243");
        assert_eq!(config.temporal_namespace, "default");
        assert_eq!(config.reconciliation_interval_secs, 5);
        assert_eq!(config.ack_timeout_secs, 30);
        assert_eq!(config.ack_retry_interval_secs, 30);
        assert_eq!(config.stale_threshold_secs, 300);
        assert!(config.database_url.starts_with("sqlite://"));
    }

    #[test]
    fn test_config_merge_empty_fields() {
        let defaults = Config::default();
        let mut config = Config {
            machine_id: String::new(),
            lantern_dir: PathBuf::new(),
            data_dir: PathBuf::new(),
            config_dir: PathBuf::new(),
            logs_dir: PathBuf::new(),
            run_dir: PathBuf::new(),
            database_url: String::new(),
            temporal_address: String::new(),
            temporal_namespace: String::new(),
            relay_socket_path: PathBuf::new(),
            relay_pid_path: PathBuf::new(),
            reconciliation_interval_secs: 0,
            ack_timeout_secs: 0,
            ack_retry_interval_secs: 0,
            stale_threshold_secs: 0,
        };

        // Simulate merge logic from load()
        if config.machine_id.is_empty() {
            config.machine_id = defaults.machine_id.clone();
        }
        if config.lantern_dir.as_os_str().is_empty() {
            config.lantern_dir = defaults.lantern_dir.clone();
        }
        if config.data_dir.as_os_str().is_empty() {
            config.data_dir = defaults.data_dir.clone();
        }
        if config.config_dir.as_os_str().is_empty() {
            config.config_dir = defaults.config_dir.clone();
        }
        if config.logs_dir.as_os_str().is_empty() {
            config.logs_dir = defaults.logs_dir.clone();
        }
        if config.run_dir.as_os_str().is_empty() {
            config.run_dir = defaults.run_dir.clone();
        }
        if config.database_url.is_empty() {
            config.database_url = defaults.database_url.clone();
        }
        if config.temporal_address.is_empty() {
            config.temporal_address = defaults.temporal_address.clone();
        }
        if config.temporal_namespace.is_empty() {
            config.temporal_namespace = defaults.temporal_namespace.clone();
        }
        if config.relay_socket_path.as_os_str().is_empty() {
            config.relay_socket_path = defaults.relay_socket_path.clone();
        }
        if config.relay_pid_path.as_os_str().is_empty() {
            config.relay_pid_path = defaults.relay_pid_path.clone();
        }
        if config.reconciliation_interval_secs == 0 {
            config.reconciliation_interval_secs = defaults.reconciliation_interval_secs;
        }
        if config.ack_timeout_secs == 0 {
            config.ack_timeout_secs = defaults.ack_timeout_secs;
        }
        if config.ack_retry_interval_secs == 0 {
            config.ack_retry_interval_secs = defaults.ack_retry_interval_secs;
        }
        if config.stale_threshold_secs == 0 {
            config.stale_threshold_secs = defaults.stale_threshold_secs;
        }

        assert_eq!(config.machine_id, defaults.machine_id);
        assert_eq!(config.temporal_address, "127.0.0.1:8243");
        assert_eq!(config.reconciliation_interval_secs, 5);
    }
}
