use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Error;
use clap::Clap;
use config::{Config, File, FileFormat};
use http::uri::PathAndQuery;

#[cfg(feature = "graphql-transport")]
use relay_ton::transport::graphql_transport::Config as TonGraphQLConfig;
#[cfg(feature = "tonlib-transport")]
use relay_ton::transport::tonlib_transport::Config as TonTonlibConfig;

use crate::prelude::*;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TonAddress(pub String);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RelayConfig {
    /// Listen address of relay. Used by the client to perform all maintenance actions.
    pub listen_address: SocketAddr,

    /// Path to json, where ton and eth private keys will be stored in encrypted way.
    pub keys_path: PathBuf,

    /// Path to Sled database.
    pub storage_path: PathBuf,

    /// Logger settings
    #[serde(default = "default_logger_settings")]
    pub logger_settings: serde_yaml::Value,

    /// Metrics settings
    #[serde(default)]
    pub metrics_settings: Option<MetricsSettings>,

    /// ETH specific settings
    pub eth_settings: EthSettings,

    /// TON specific settings
    pub ton_settings: TonSettings,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MetricsSettings {
    /// Listen address of metrics. Used by the client to gather prometheus metrics
    pub listen_address: SocketAddr,

    /// Listen address of metrics. Used by the client to gather prometheus metrics
    #[serde(default = "default_metrics_path", with = "relay_utils::serde_url")]
    pub metrics_path: PathAndQuery,

    /// Metrics poll interval
    #[serde(with = "relay_utils::serde_time")]
    pub collection_interval: Duration,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EthSettings {
    /// Address of ethereum node. Only http is supported right now
    pub node_address: String,

    /// Number of concurrent tcp connection to ethereum node
    pub tcp_connection_count: usize,

    /// Timeout and delay between retries for getting non-critical data, like current eth sync status
    #[serde(with = "relay_utils::serde_time")]
    pub get_eth_data_timeout: Duration,

    /// Number of attempts  for getting non-critical data, like current eth sync status
    pub get_eth_data_attempts: u64,

    /// Poll interval between fetching new blocks
    #[serde(with = "relay_utils::serde_time")]
    pub eth_poll_interval: Duration,

    /// Number of attempts to get logs in the block
    pub eth_poll_attempts: u64,

    /// Offset in blocks for checking suspicious transactions
    pub suspicious_blocks_offset: u64,

    /// Bridge address in ethereum
    pub bridge_address: Address,
}

impl Default for EthSettings {
    fn default() -> Self {
        Self {
            node_address: "http://localhost:8545".into(),
            tcp_connection_count: 100,
            get_eth_data_timeout: Duration::from_secs(10),
            get_eth_data_attempts: 50,
            eth_poll_interval: Duration::from_secs(10),
            eth_poll_attempts: 86400 / 10,
            suspicious_blocks_offset: 10,
            bridge_address: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TonSettings {
    /// Relay account address
    pub relay_contract_address: TonAddress,

    /// Bridge contract address
    pub bridge_contract_address: TonAddress,

    /// TON transport config
    pub transport: TonTransportConfig,

    /// Interval between attempts to get event configuration details
    #[serde(with = "relay_utils::serde_time")]
    pub event_configuration_details_retry_interval: Duration,
    /// Amount of unsuccessful attempts
    pub event_configuration_details_retry_count: u64,

    /// Interval between attempts to get event details
    #[serde(with = "relay_utils::serde_time")]
    pub event_details_retry_interval: Duration,
    /// Amount of unsuccessful attempts
    pub event_details_retry_count: u64,

    /// Interval between attempts to get send message
    #[serde(with = "relay_utils::serde_time")]
    pub message_retry_interval: Duration,
    /// Amount of unsuccessful attempts
    pub message_retry_count: u64,
    /// Coefficient, on which every interval will be multiplied
    pub message_retry_interval_multiplier: f64,

    /// Amount of parallel sent messages in ton
    pub parallel_spawned_contracts_limit: usize,

    /// TON events verification interval
    #[serde(with = "relay_utils::serde_time")]
    pub ton_events_verification_interval: Duration,

    /// TON events verification queue logical time offset
    pub ton_events_verification_queue_lt_offset: u64,
    /// TON events allowed timestamp difference
    pub ton_events_allowed_time_diff: u32,

    /// Amount of attempts to create events handler
    pub events_handler_retry_count: u64,
    /// Duration between attempts to create events handler
    #[serde(with = "relay_utils::serde_time")]
    pub events_handler_interval: Duration,
}

impl Default for TonSettings {
    fn default() -> Self {
        Self {
            relay_contract_address: Default::default(),
            bridge_contract_address: Default::default(),
            transport: TonTransportConfig::default(),
            event_configuration_details_retry_count: 100,
            event_configuration_details_retry_interval: Duration::from_secs(5),
            event_details_retry_interval: Default::default(),
            event_details_retry_count: 100,
            message_retry_interval: Duration::from_secs(60),
            message_retry_count: 10,
            message_retry_interval_multiplier: 1.5,
            parallel_spawned_contracts_limit: 10,
            ton_events_verification_interval: Duration::from_secs(1),
            ton_events_verification_queue_lt_offset: 10,
            ton_events_allowed_time_diff: 10,
            events_handler_retry_count: 50,
            events_handler_interval: Duration::from_secs(10),
        }
    }
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            keys_path: PathBuf::from("/var/lib/relay/keys.json"),
            listen_address: "127.0.0.1:12345".parse().unwrap(),
            storage_path: PathBuf::from("/var/lib/relay/persistent_storage"),
            logger_settings: default_logger_settings(),
            metrics_settings: Some(MetricsSettings {
                listen_address: "127.0.0.1:10000".parse().unwrap(),
                metrics_path: default_metrics_path(),
                collection_interval: Duration::from_secs(10),
            }),
            eth_settings: EthSettings::default(),
            ton_settings: TonSettings::default(),
        }
    }
}

fn default_logger_settings() -> serde_yaml::Value {
    const DEFAULT_LOG4RS_SETTINGS: &str = r##"
    appenders:
      stdout:
        kind: console
        encoder:
          pattern: "{d(%Y-%m-%d %H:%M:%S %Z)(utc)} - {h({l})} {M} {f}:{L} = {m} {n}"
    root:
      level: error
      appenders:
        - stdout
    loggers:
      relay:
        level: info
        appenders:
          - stdout
        additive: false
      relay_eth:
        level: info
        appenders:
          - stdout
        additive: false
      relay_ton:
        level: info
        appenders:
          - stdout
        additive: false
    "##;
    serde_yaml::from_str(DEFAULT_LOG4RS_SETTINGS).unwrap()
}

fn default_metrics_path() -> PathAndQuery {
    PathAndQuery::from_static("/")
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TonTransportConfig {
    #[cfg(feature = "tonlib-transport")]
    Tonlib(TonTonlibConfig),
    #[cfg(feature = "graphql-transport")]
    GraphQl(TonGraphQLConfig),
}

#[cfg(any(feature = "tonlib-transport", feature = "graphql-transport"))]
impl Default for TonTransportConfig {
    fn default() -> Self {
        #[cfg(feature = "tonlib-transport")]
        return Self::Tonlib(TonTonlibConfig::default());

        #[cfg(all(feature = "graphql-transport", not(feature = "tonlib-transport")))]
        Self::GraphQl(TonGraphQLConfig::default())
    }
}

pub fn read_config(path: PathBuf) -> Result<RelayConfig, Error> {
    let mut config = Config::new();
    config.merge(File::from(path).format(FileFormat::Yaml))?;
    config.merge(config::Environment::new())?;

    let config: RelayConfig = config.try_into()?;
    Ok(config)
}

pub fn read_env() -> Result<RelayConfig, Error> {
    let config_string = std::env::var("config_string")
        .map_err(|e| anyhow::Error::new(e).context("config_string not set"))?;
    let config = serde_json::from_str(&config_string)?;
    Ok(config)
}

#[derive(Deserialize, Serialize, Clone, Debug, Clap)]
pub struct Arguments {
    /// Path to config
    #[clap(short, long, conflicts_with = "gen-config")]
    pub config: Option<PathBuf>,

    /// Generate default config
    #[clap(long)]
    pub gen_config: Option<PathBuf>,
}

pub fn generate_config<T>(path: T) -> Result<(), Error>
where
    T: AsRef<Path>,
{
    let mut file = std::fs::File::create(path)?;
    let config = RelayConfig::default();
    file.write_all(serde_yaml::to_string(&config)?.as_bytes())?;
    Ok(())
}

pub fn parse_args() -> Arguments {
    Arguments::parse()
}
