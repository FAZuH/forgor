use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use log::debug;
use log::info;
use serde::Deserialize;
use serde::Serialize;

use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub pomodoro: PomodoroConfig,
    pub logs_path: PathBuf,
    pub db_path: PathBuf,
    #[serde(skip)]
    pub conf_path: PathBuf,
    #[serde(skip)]
    pub conf_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(utils::conf_dir())
    }
}

impl Config {
    pub fn new(conf_dir: PathBuf) -> Self {
        let logs_path = conf_dir.join("logs");
        let db_path = conf_dir.join("data.db");
        let conf_path = conf_dir.join("config.yaml");

        Self {
            pomodoro: Default::default(),
            logs_path,
            db_path,
            conf_path,
            conf_dir,
        }
    }

    pub fn load(&mut self) -> Result<(), ConfigError> {
        let conf_dir = &self.conf_dir;
        let conf_path = &self.conf_path;

        debug!("config directory: {:?}", conf_dir);
        if !conf_dir.exists() {
            fs::create_dir_all(conf_dir)?;
            info!("created config directory at {conf_dir:?}");
        }

        let config = if !conf_path.exists() {
            let config = Config::default();
            let file = fs::File::create(conf_path)?;
            serde_norway::to_writer(&file, &config)?;
            info!("written default config to {:?}", conf_path);
            config
        } else {
            let file = fs::File::open(conf_path)?;
            let config = serde_norway::from_reader(&file)?;
            info!("loaded configuration");
            config
        };
        let conf_dir = self.conf_dir.clone();
        let conf_path = self.conf_path.clone();
        *self = config;
        self.conf_dir = conf_dir;
        self.conf_path = conf_path;
        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let file = fs::File::create(&self.conf_path)?;
        serde_norway::to_writer(&file, self)?;
        info!("Configuration saved successfully");
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Percentage(f32);

impl Percentage {
    pub fn new(perc: f32) -> Self {
        let mut _self = Self::muted();
        _self.set_clamp(perc);
        _self
    }

    pub fn set(&mut self, perc: f32) {
        self.0 = perc
    }

    pub fn set_clamp(&mut self, perc: f32) {
        self.0 = perc.clamp(0.0, 1.0)
    }

    pub fn muted() -> Self {
        Self(0.0)
    }

    pub fn half() -> Self {
        Self(0.5)
    }

    pub fn full() -> Self {
        Self(1.0)
    }

    pub fn volume(&self) -> f32 {
        self.0
    }
}

impl Default for Percentage {
    fn default() -> Self {
        Self::half()
    }
}

impl TryFrom<&str> for Percentage {
    type Error = std::num::ParseIntError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let f: i32 = value.to_string().parse()?;
        Ok(Percentage::new(f as f32 / 100.0))
    }
}

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_norway::Error),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PomodoroConfig {
    pub timer: Timers,
    pub hook: Hooks,
    pub alarm: Alarms,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Timers {
    pub auto_start_on_launch: bool,
    #[serde(with = "duration_as_secs")]
    pub focus: Duration,
    #[serde(with = "duration_as_secs")]
    pub short: Duration,
    #[serde(with = "duration_as_secs")]
    pub long: Duration,

    pub long_interval: u32,

    pub auto_focus: bool,
    pub auto_short: bool,
    pub auto_long: bool,
}

mod duration_as_secs {
    use serde::Deserializer;
    use serde::Serializer;

    use super::*;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

impl Default for Timers {
    fn default() -> Self {
        Self {
            auto_start_on_launch: true,
            focus: Duration::from_mins(25),
            short: Duration::from_mins(5),
            long: Duration::from_mins(10),
            long_interval: 4,
            auto_focus: false,
            auto_short: false,
            auto_long: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Hooks {
    pub focus: String,
    pub short: String,
    pub long: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Alarms {
    pub focus: Alarm,
    pub short: Alarm,
    pub long: Alarm,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Alarm {
    pub path: Option<PathBuf>,
    pub volume: crate::config::Percentage,
}

impl Alarm {
    pub fn volume(&self) -> String {
        self.volume.to_string()
    }

    pub fn path(&self) -> String {
        self.path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    }
}
