use std::pin::Pin;
use std::task::{Context, Poll};

use super::errors::*;
use super::prelude::*;
use crate::prelude::*;
use crate::transport::*;

pub async fn make_ton_swapback_contract(
    transport: Arc<dyn Transport>,
    account: MsgAddressInt,
    event_abi: String,
) -> ContractResult<TonSwapBackEvents> {
    let event_abi: SwapBackEventAbi =
        serde_json::from_str(&event_abi).map_err(|_| ContractError::InvalidAbi)?;

    let mut abi = AbiEvent {
        abi_version: 2,
        name: event_abi.name,
        inputs: event_abi.inputs,
        id: 0,
    };
    abi.id = if let Some(id) = event_abi.id {
        id
    } else {
        abi.get_function_id() & 0x7FFFFFFF
    };

    let (subscription, events_rx) = transport.subscribe_full(account.clone()).await?;

    Ok(TonSwapBackEvents {
        account,
        abi: Arc::new(abi),
        subscription,
        events_rx,
    })
}

pub struct TonSwapBackEvents {
    account: MsgAddressInt,
    abi: Arc<AbiEvent>,
    subscription: Arc<dyn AccountSubscription>,
    events_rx: FullEventsRx,
}

impl TonSwapBackEvents {
    pub fn address(&self) -> &MsgAddressInt {
        &self.account
    }

    pub fn since_lt(&self) -> u64 {
        self.subscription.since_lt()
    }

    pub fn abi(&self) -> &Arc<AbiEvent> {
        &self.abi
    }

    pub fn get_known_events(
        &self,
        since_lt: Option<u64>,
        until_lt: u64,
    ) -> BoxStream<'_, Vec<Token>> {
        self.subscription
            .rescan_events(since_lt, Some(until_lt + 1))
            .filter_map(move |raw_event| async move {
                match raw_event {
                    Ok(raw_event) => match self.abi.decode_input(raw_event) {
                        Ok(tokens) => Some(tokens),
                        Err(e) => {
                            log::debug!("Skipping unknown swapback event: {}", e.to_string());
                            None
                        }
                    },
                    Err(e) => {
                        log::error!("Failed to get known swapback events: {:?}", e);
                        None
                    }
                }
            })
            .boxed()
    }
}

impl Stream for TonSwapBackEvents {
    type Item = SwapBackEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.events_rx.poll_recv(cx) {
                Poll::Ready(Some(raw_event)) => match self.abi.decode_input(raw_event.event_data) {
                    Ok(tokens) => {
                        return Poll::Ready(Some(SwapBackEvent {
                            event_transaction: raw_event.event_transaction,
                            event_transaction_lt: raw_event.event_transaction_lt,
                            event_index: raw_event.event_index,
                            tokens,
                        }))
                    }
                    Err(e) => {
                        log::debug!("Skipping unknown swapback event: {}", e.to_string());
                    }
                },
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapBackEvent {
    pub event_transaction: UInt256,
    pub event_transaction_lt: u64,
    pub event_index: u64,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwapBackEventAbi {
    name: String,

    #[serde(default)]
    inputs: Vec<ton_abi::Param>,

    #[serde(default)]
    #[serde(deserialize_with = "ton_abi::contract::deserialize_opt_u32_from_string")]
    id: Option<u32>,
}
