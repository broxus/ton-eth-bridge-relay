use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::Error;
use clap::Clap;
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use sha3::Digest;

use relay_eth::ws::{Address as EthAddr, H256};
use relay_ton::transport::tonlib_transport::Config as TonConfig;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct EthAddress(String);

#[derive(Deserialize, Serialize, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct TonAddress(pub String);

impl EthAddress {
    pub fn to_eth_addr(&self) -> Result<EthAddr, Error> {
        let bytes = hex::decode(&self.0)?;
        let hash = sha3::Keccak256::digest(&*bytes);
        Ok(EthAddr::from_slice(&*hash))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Method(String);

impl Method {
    pub fn to_topic_hash(&self) -> Result<H256, Error> {
        let bytes = hex::decode(&self.0)?;
        let hash = sha3::Keccak256::digest(&*bytes);
        Ok(H256::from_slice(&*hash))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RelayConfig {
    pub encrypted_data: PathBuf,
    pub eth_node_address: String,
    pub ton_contract_address: TonAddress,
    pub storage_path: PathBuf,
    pub listen_address: SocketAddr,
    pub ton_config: TonConfig,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            encrypted_data: PathBuf::from("./cryptodata.json"),
            storage_path: PathBuf::from("./persistent_storage"),
            eth_node_address: "ws://localhost:12345".into(),
            ton_contract_address: TonAddress("".into()),
            listen_address: "127.0.0.1:12345".parse().unwrap(),
            ton_config: TonConfig::default(),
        }
    }
}

pub fn read_config(path: &str) -> Result<RelayConfig, Error> {
    let mut config = Config::new();
    config.merge(File::new(path, FileFormat::Json))?;
    let config: RelayConfig = config.try_into()?;
    Ok(config)
}

#[derive(Deserialize, Serialize, Clone, Debug, Clap)]
pub struct Arguments {
    #[clap(short, long, default_value = "config.json", conflicts_with = "gen-config")]
    pub config: String,
    ///It will generate default config
    #[clap(long, requires = "crypto-store-path")]
    pub gen_config: bool,
    #[clap(long)]
    /// Path for encrypted data storage
    pub crypto_store_path: Option<PathBuf>,
    ///Path for generated config
    #[clap(long, default_value = "default_config.json")]
    pub generated_config_path: PathBuf,
}

pub fn generate_config<T>(path: T, pem_path: PathBuf) -> Result<(), Error>
where
    T: AsRef<Path>,
{
    let mut file = std::fs::File::create(path)?;
    let mut config = RelayConfig::default();
    config = RelayConfig {
        encrypted_data: pem_path,
        ..config
    };
    file.write_all(serde_json::to_vec_pretty(&config)?.as_slice())?;
    Ok(())
}

pub fn parse_args() -> Arguments {
    dbg!(Arguments::parse())
}
