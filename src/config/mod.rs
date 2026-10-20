//! Runtime configuration.
//!
//! Load order (later wins):
//! 1. embedded defaults (`config/default.toml` via `include_str!` fallback values)
//! 2. optional user file (`$XDG_CONFIG_HOME/ircchannel/config.toml`)
//! 3. environment variables (`IRC_HOST`, `IRC_PORT`, `IRC_NICK`, …)

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const DEFAULT_TOML: &str = include_str!("../../config/default.toml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub connection: ConnectionConfig,
    pub identity: IdentityConfig,
    pub session: SessionConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub server_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub nick: String,
    pub username: String,
    pub realname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub send_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_tick_ms")]
    pub tick_ms: u64,
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
    /// IANA name, default America/Mexico_City.
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_tick_ms() -> u64 {
    250
}

fn default_history_limit() -> usize {
    2000
}

fn default_timezone() -> String {
    "America/Mexico_City".into()
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_TOML).expect("embedded default.toml must parse")
    }
}

impl Config {
    /// Load config using the documented precedence chain.
    pub fn load() -> Result<Self> {
        let mut cfg = Config::default();

        if let Some(path) = user_config_path() {
            if path.is_file() {
                let raw = fs::read_to_string(&path)
                    .map_err(|e| Error::Config(format!("read {}: {e}", path.display())))?;
                let file_cfg: Config = toml::from_str(&raw)
                    .map_err(|e| Error::Config(format!("parse {}: {e}", path.display())))?;
                cfg = file_cfg;
            }
        }

        cfg.apply_env();
        cfg.validate()?;
        Ok(cfg)
    }

    fn apply_env(&mut self) {
        if let Ok(host) = std::env::var("IRC_HOST") {
            self.connection.host = host;
        }
        if let Ok(port) = std::env::var("IRC_PORT") {
            if let Ok(p) = port.parse() {
                self.connection.port = p;
            }
        }
        if let Ok(pass) = std::env::var("IRC_PASSWORD") {
            self.connection.server_password = pass;
        }
        if let Ok(nick) = std::env::var("IRC_NICK") {
            self.identity.nick = nick;
        }
        if let Ok(user) = std::env::var("IRC_USER") {
            self.identity.username = user;
        }
        if let Ok(real) = std::env::var("IRC_REALNAME") {
            self.identity.realname = real;
        }
        if let Ok(channels) = std::env::var("IRC_CHANNELS") {
            self.session.channels = channels
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(normalize_channel)
                .collect();
        }
        if let Ok(tz) = std::env::var("IRC_TZ") {
            if !tz.trim().is_empty() {
                self.ui.timezone = tz;
            }
        }
    }

    fn validate(&self) -> Result<()> {
        if self.connection.host.trim().is_empty() {
            return Err(Error::Config("connection.host is empty".into()));
        }
        if self.identity.nick.trim().is_empty() {
            return Err(Error::Config("identity.nick is empty".into()));
        }
        if self.connection.port == 0 {
            return Err(Error::Config("connection.port must be non-zero".into()));
        }
        if self.ui.timezone.trim().is_empty() {
            return Err(Error::Config("ui.timezone is empty".into()));
        }
        // Fail fast if the IANA name is unknown.
        if crate::timefmt::parse_timezone(&self.ui.timezone).is_err() {
            return Err(Error::Config(format!(
                "ui.timezone {:?} is not a valid IANA timezone",
                self.ui.timezone
            )));
        }
        Ok(())
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.connection.host, self.connection.port)
    }
}

pub fn user_config_path() -> Option<PathBuf> {
    ProjectDirs::from("mx", "ditra", "ircchannel").map(|d| d.config_dir().join("config.toml"))
}

pub fn normalize_channel(name: &str) -> String {
    let t = name.trim();
    if t.starts_with('#') || t.starts_with('&') {
        t.to_string()
    } else {
        format!("#{t}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_toml_parses() {
        let cfg = Config::default();
        assert_eq!(cfg.connection.host, "irc.example.com");
        assert_eq!(cfg.connection.port, 6667);
        assert_eq!(cfg.session.channels, vec!["#general".to_string()]);
        assert_eq!(cfg.ui.timezone, "America/Mexico_City");
    }

    #[test]
    fn normalize_channel_adds_hash() {
        assert_eq!(normalize_channel("ditra"), "#ditra");
        assert_eq!(normalize_channel("#ops"), "#ops");
    }
}
