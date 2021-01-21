use opg::OpgModel;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, OpgModel)]
pub struct InitData {
    pub ton_seed: String,
    pub eth_seed: String,
    pub password: String,
    pub language: String,
    pub ton_derivation_path: Option<String>,
    pub eth_derivation_path: Option<String>,
}

#[derive(Deserialize, Debug, OpgModel, Serialize)]
pub struct Password {
    pub password: String,
}

#[derive(Deserialize, Debug, Serialize, OpgModel)]
pub struct RescanEthData {
    pub block: u64,
}

#[derive(Deserialize, Serialize, OpgModel)]
pub struct NewEventConfiguration {
    pub configuration_id: String,
    pub address: String,
    pub configuration_type: EventConfigurationType,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, OpgModel)]
#[serde(rename_all = "lowercase")]
pub enum EventConfigurationType {
    Eth,
    Ton,
}

#[derive(Deserialize, Serialize)]
pub struct EventConfiguration {
    pub configuration_id: String,
    pub ethereum_event_abi: String,
    pub ethereum_event_address: String,
    pub event_proxy_address: String,
    pub ethereum_event_blocks_to_confirm: u16,
    pub event_required_confirmations: u64,
    pub event_required_rejects: u64,
    pub event_initial_balance: u64,
    pub bridge_address: String,
    pub event_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, OpgModel)]
pub struct VotingAddress {
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, OpgModel)]
#[serde(rename_all = "lowercase", tag = "vote", content = "address")]
pub enum Voting {
    Confirm(String),
    Reject(String),
}

#[derive(Serialize, Deserialize, OpgModel)]
pub struct Status {
    pub password_needed: bool,
    pub init_data_needed: bool,
    pub is_working: bool,
    pub ton_relay_address: Option<String>,
    pub eth_pubkey: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, opg::OpgModel)]
pub struct EthTonVoteView {
    pub event_address: String,
    #[serde(flatten)]
    pub transaction: EthTonTransactionView,
}

#[derive(Serialize, Deserialize, Clone, opg::OpgModel)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum EthTonTransactionView {
    Confirm(EthEventVoteDataView),
    Reject(EthEventVoteDataView),
}

#[derive(Serialize, Deserialize, Clone, opg::OpgModel)]
pub struct TonEthVoteView {
    pub event_address: String,
    #[serde(flatten)]
    pub transaction: TonEthTransactionView,
}

#[derive(Serialize, Deserialize, Clone, opg::OpgModel)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum TonEthTransactionView {
    Confirm(SignedVoteDataView),
    Reject(TonEventVoteDataView),
}

#[derive(Serialize, Deserialize, Clone, opg::OpgModel)]
pub struct SignedVoteDataView {
    pub signature: String,
    pub data: TonEventVoteDataView,
}

#[derive(Serialize, Deserialize, Clone, opg::OpgModel)]
pub struct TonEventVoteDataView {
    pub configuration_id: String,
    pub event_transaction: String,
    pub event_transaction_lt: u64,
    pub event_index: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, opg::OpgModel)]
pub struct EthEventVoteDataView {
    #[opg(format = "hex")]
    pub event_transaction: String,
    pub event_index: u32,
    #[opg(format = "hex")]
    pub event_data: String,
    pub event_block_number: u64,
    #[opg(format = "hex")]
    pub event_block: String,
    pub configuration_id: String,
}

#[derive(Deserialize, Serialize, opg::OpgModel)]
#[serde(rename_all = "lowercase")]
pub struct EthTxStatView {
    pub tx_hash: String,
    #[opg("Timestamp in seconds")]
    pub met: String,
    pub event_addr: String,
    pub vote: EventVote,
}

#[derive(Deserialize, Serialize, opg::OpgModel)]
pub struct TonTxStatView {
    pub tx_hash: String,
    pub tx_lt: String,
    #[opg("Timestamp in seconds")]
    pub met: String,
    pub event_addr: String,
    pub vote: EventVote,
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize, OpgModel)]
#[serde(rename_all = "lowercase")]
pub enum EventVote {
    Confirm,
    Reject,
}
