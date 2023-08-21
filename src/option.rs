use std::fmt::Display;
use serde::{Deserialize, Serialize};
use std::{fmt, env};
use std::ffi::OsStr;
use clap::Parser;
use std::str::FromStr;
use std::path::PathBuf;
use std::env::VarError;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug)]
pub struct LogLevelError {
    pub given_log_level: String,
}

impl Display for LogLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Log level '{}' is invalid. Accepted values are 'OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', and 'TRACE'.",
            self.given_log_level
        )
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Off => Display::fmt("OFF", f),
            LogLevel::Error => Display::fmt("ERROR", f),
            LogLevel::Warn => Display::fmt("WARN", f),
            LogLevel::Info => Display::fmt("INFO", f),
            LogLevel::Debug => Display::fmt("DEBUG", f),
            LogLevel::Trace => Display::fmt("TRACE", f),
        }
    }
}

impl std::error::Error for LogLevelError {}

impl FromStr for LogLevel {
    type Err = LogLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "off" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(LogLevelError { given_log_level: s.to_owned() }),
        }
    }
}


const STREAMKIT_LOG_LEVEL: &str = "STREAMKIT_LOG_LEVEL";
const STREAMKIT_ENABLE_METRICS: &str = "STREAMKIT_ENABLE_METRICS";
const STREAMKIT_PART_SIZE: &str = "STREAMKIT_ENABLE_METRICS";
const STREAMKIT_WINDOW_SIZE: &str = "STREAMKIT_ENABLE_METRICS";

const DEFAULT_CONFIG_FILE_PATH: &str = "./config.toml";

#[derive(Debug, Clone, Parser, Deserialize)]
#[clap(version, next_display_order = None)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Opt {
    /// Defines how much detail should be present in StreamKit's logs.
    ///
    /// StreamKit currently supports six log levels, listed in order of increasing verbosity: OFF, ERROR, WARN, INFO, DEBUG, TRACE.
    #[clap(long, env = STREAMKIT_LOG_LEVEL, default_value_t)]
    #[serde(default)]
    pub log_level: LogLevel,

    /// Experimental metrics feature.
    ///
    /// Enables the Prometheus metrics on the `GET /metrics` endpoint.
    #[clap(long, env = STREAMKIT_ENABLE_METRICS)]
    #[serde(default)]
    pub enable_metrics: bool,

    /// Set the path to a configuration file that should be used to setup the engine.
    /// Format must be TOML.
    #[clap(long)]
    pub config_file_path: Option<PathBuf>,

    //Sets the size of the partials that make up the fmp4 segments
    #[clap(long, env = STREAMKIT_PART_SIZE, default_value_t = 10)]
    #[serde(default)]
    pub part_duration: f32,

    //Sets the windows size (rewind window) for the HLS stream.
    #[clap(long, env = STREAMKIT_WINDOW_SIZE, default_value_t = 15)]
    #[serde(default)]
    pub window_size: usize,
}


impl Opt {
    /// Build a new Opt from config file, env vars and cli args.
    pub fn try_build() -> anyhow::Result<(Self, Option<PathBuf>)> {
        // Parse the args to get the config_file_path.
        let mut opts = Opt::parse();
        let mut config_read_from = None;
        let user_specified_config_file_path = opts
            .config_file_path
            .clone()
            .or_else(|| env::var("CONFIG_FILE_PATH").map(PathBuf::from).ok());
        let config_file_path = user_specified_config_file_path
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILE_PATH));

        match std::fs::read_to_string(&config_file_path) {
            Ok(config) => {
                // If the file is successfully read, we deserialize it with `toml`.
                let opt_from_config = toml::from_str::<Opt>(&config)?;
                // Return an error if config file contains 'config_file_path'
                // Using that key in the config file doesn't make sense bc it creates a logical loop (config file referencing itself)
                if opt_from_config.config_file_path.is_some() {
                    anyhow::bail!("`config_file_path` is not supported in the configuration file")
                }
                // We inject the values from the toml in the corresponding env vars if needs be. Doing so, we respect the priority toml < env vars < cli args.
                opt_from_config.export_to_env();
                // Once injected we parse the cli args once again to take the new env vars into scope.
                opts = Opt::parse();
                config_read_from = Some(config_file_path);
            }
            Err(e) => {
                if let Some(path) = user_specified_config_file_path {
                    // If we have an error while reading the file defined by the user.
                    anyhow::bail!(
                        "unable to open or read the {:?} configuration file: {}.",
                        path,
                        e,
                    )
                }
            }
        }

        Ok((opts, config_read_from))
    }

    fn export_to_env(self) {
        let Opt {
            log_level,
            enable_metrics: enable_metrics_route,
            config_file_path: _,
            part_duration,
            window_size: _,
        } = self;

        export_to_env_if_not_present(STREAMKIT_LOG_LEVEL, log_level.to_string());
        export_to_env_if_not_present(STREAMKIT_ENABLE_METRICS,enable_metrics_route.to_string());
        export_to_env_if_not_present(STREAMKIT_PART_SIZE,part_duration.to_string());
        //export_to_env_if_not_present(STREAMKIT_WINDOW_SIZE, window_size.to_string());
    }
}

pub fn export_to_env_if_not_present<T>(key: &str, value: T)
where
    T: AsRef<OsStr>,
{
    if let Err(VarError::NotPresent) = std::env::var(key) {
        std::env::set_var(key, value);
    }
}