use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Transport type for data sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    #[default]
    Local,
    Ssh,
}

impl std::fmt::Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transport::Local => write!(f, "local"),
            Transport::Ssh => write!(f, "ssh"),
        }
    }
}

/// A path with an associated transport type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    #[serde(default)]
    pub transport: Transport,
    pub path: String,
}

impl PathConfig {
    pub fn is_set(&self) -> bool {
        !self.path.is_empty()
    }
}

/// SSH connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: i32,
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_path: Option<String>,
}

impl SshConfig {
    pub fn is_configured(&self) -> bool {
        !self.host.is_empty() && !self.user.is_empty()
    }
}

/// Default paths for each service, organized by platform.
pub struct DefaultPaths;

impl DefaultPaths {
    pub fn steam() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let platform = std::env::consts::OS;

        match platform {
            "linux" => vec![
                home.join(".local/share/Steam"),
                home.join("Steam"),
                PathBuf::from("/usr/local/share/steam"),
                PathBuf::from("/usr/share/steam"),
            ],
            "macos" => vec![home.join("Library/Application Support/Steam")],
            "windows" => vec![
                PathBuf::from("C:\\Program Files\\Steam"),
                PathBuf::from("C:\\Program Files (x86)\\Steam"),
            ],
            _ => vec![home.join(".local/share/Steam")],
        }
    }

    pub fn egs() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let platform = std::env::consts::OS;

        match platform {
            "linux" => vec![
                home.join(".local/share/Epic Games"),
                home.join(".config/Epic/EpicGamesLauncher"),
                PathBuf::from("/var/run/epicgameslauncher/Manifests"),
            ],
            "macos" => vec![home.join("Library/Application Support/Epic")],
            "windows" => vec![
                PathBuf::from("C:\\ProgramData\\Epic\\EpicGamesLauncher\\Data\\Manifests"),
                PathBuf::from("C:\\Program Files\\Epic Games\\Launcher\\Manifests"),
            ],
            _ => vec![home.join(".local/share/EpicGamesLauncher/Manifests")],
        }
    }

    pub fn sunshine() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        vec![
            home.join(".config/sunshine"),
            home.join(".var/app/dev.lizardbyte.app.Sunshine/config/sunshine"),
            PathBuf::from("/etc/sunshine"),
        ]
    }

    pub fn lutris() -> Vec<PathBuf> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        vec![
            home.join(".local/share/lutris"),
            home.join(".var/app/com.lutris.Lutris/config/lutris"),
        ]
    }
}

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "sunshine_config_dir")]
    pub sunshine_config_dir: Option<PathConfig>,
    #[serde(rename = "sunshine_config_path")]
    pub sunshine_config_path: Option<String>,
    #[serde(rename = "apps_json_path")]
    pub apps_json_path: Option<String>,
    #[serde(rename = "steam_path")]
    pub steam_path: Option<PathConfig>,
    #[serde(rename = "egs_manifest_path")]
    pub egs_manifest_path: Option<PathConfig>,
    #[serde(rename = "egs_install_path")]
    pub egs_install_path: Option<PathConfig>,
    #[serde(rename = "lutris_path")]
    pub lutris_path: Option<PathConfig>,
    pub plugins: Vec<String>,
    #[serde(rename = "dry_run")]
    pub dry_run: bool,
    pub language: String,
    #[serde(rename = "backup_suffix")]
    pub backup_suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshConfig>,
    #[serde(rename = "steamgrid_api_key")]
    pub steamgrid_api_key: Option<String>,
    #[serde(rename = "fetch_metadata")]
    pub fetch_metadata: bool,
    #[serde(rename = "use_flat_spawn")]
    pub use_flat_spawn: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            sunshine_config_dir: None,
            sunshine_config_path: None,
            apps_json_path: None,
            steam_path: None,
            egs_manifest_path: None,
            egs_install_path: None,
            lutris_path: None,
            plugins: vec!["steam".to_string(), "egs".to_string()],
            dry_run: false,
            language: "en".to_string(),
            backup_suffix: None,
            ssh: None,
            steamgrid_api_key: None,
            fetch_metadata: false,
            use_flat_spawn: false,
        }
    }
}

impl AppConfig {
    pub fn sunshine_path(&self) -> (Option<String>, Transport) {
        self.sunshine_config_dir
            .as_ref()
            .map(|pc| (Some(pc.path.clone()), pc.transport.clone()))
            .unwrap_or_else(|| (None, Transport::Local))
    }

    pub fn steam_path(&self) -> (Option<String>, Transport) {
        self.steam_path
            .as_ref()
            .map(|pc| (Some(pc.path.clone()), pc.transport.clone()))
            .unwrap_or_else(|| (None, Transport::Local))
    }

    pub fn egs_manifest_path(&self) -> (Option<String>, Transport) {
        self.egs_manifest_path
            .as_ref()
            .map(|pc| (Some(pc.path.clone()), pc.transport.clone()))
            .unwrap_or_else(|| (None, Transport::Local))
    }

    pub fn egs_install_path(&self) -> (Option<String>, Transport) {
        self.egs_install_path
            .as_ref()
            .map(|pc| (Some(pc.path.clone()), pc.transport.clone()))
            .unwrap_or_else(|| (None, Transport::Local))
    }

    pub fn lutris_path(&self) -> (Option<String>, Transport) {
        self.lutris_path
            .as_ref()
            .map(|pc| (Some(pc.path.clone()), pc.transport.clone()))
            .unwrap_or_else(|| (None, Transport::Local))
    }

    pub fn backup_suffix(&self) -> String {
        self.backup_suffix.clone().unwrap_or_else(|| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            let secs = now.as_secs();
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
                .unwrap_or_else(chrono::Utc::now);
            dt.format("%Y-%m-%d_%H-%M-%S").to_string()
        })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn find_config_path() -> Option<PathBuf> {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let config = dir.join("config.yaml");
                if config.exists() {
                    return Some(config);
                }
            }
        }

        if let Some(home) = dirs::config_dir() {
            let config = home.join("sunshine-stream-apps-manager/config.yaml");
            if config.exists() {
                return Some(config);
            }
        }

        if let Some(home) = dirs::home_dir() {
            let config = home.join("config.yaml");
            if config.exists() {
                return Some(config);
            }
        }

        let config = PathBuf::from("config.yaml");
        if config.exists() {
            return Some(config);
        }

        None
    }

    pub fn load() -> Result<Self, anyhow::Error> {
        let config_path = std::env::var("SSAM_CONFIG")
            .ok()
            .map(PathBuf::from)
            .or_else(Self::find_config_path);

        let mut config = Self::default();

        if let Some(path) = config_path {
            if let Ok(file_config) = Self::from_file(&path) {
                config = file_config;
            }
        }

        if let Ok(lang) = std::env::var("SSAM_LANGUAGE") {
            config.language = lang;
        }
        if let Ok(dry) = std::env::var("SSAM_DRY_RUN") {
            config.dry_run = dry.parse().unwrap_or(false);
        }
        if let Ok(fetch) = std::env::var("SSAM_FETCH_METADATA") {
            config.fetch_metadata = fetch.parse().unwrap_or(false);
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_default() {
        assert_eq!(Transport::default(), Transport::Local);
    }

    #[test]
    fn test_path_config() {
        let pc = PathConfig {
            transport: Transport::Local,
            path: "/test/path".to_string(),
        };
        assert!(pc.is_set());

        let empty = PathConfig {
            transport: Transport::Ssh,
            path: String::new(),
        };
        assert!(!empty.is_set());
    }

    #[test]
    fn test_ssh_config() {
        let ssh = SshConfig {
            host: "192.168.1.1".to_string(),
            port: 22,
            user: "user".to_string(),
            password: None,
            key_path: Some("~/.ssh/id_rsa".to_string()),
        };
        assert!(ssh.is_configured());

        let empty = SshConfig {
            host: String::new(),
            port: 0,
            user: String::new(),
            password: None,
            key_path: None,
        };
        assert!(!empty.is_configured());
    }

    #[test]
    fn test_config_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.language, "en");
        assert!(!config.dry_run);
        assert_eq!(config.plugins, vec!["steam", "egs"]);
    }
}
