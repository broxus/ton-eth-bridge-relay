use super::errors::*;
use super::models::*;
use super::prelude::*;
use crate::models::*;
use crate::prelude::*;
use crate::transport::*;

#[derive(Clone)]
pub struct EthereumEventConfigurationContract {
    transport: Arc<dyn Transport>,
    contract: Arc<ton_abi::Contract>,
}

impl EthereumEventConfigurationContract {
    pub async fn new(transport: Arc<dyn Transport>) -> ContractResult<Self> {
        let contract = Arc::new(
            ton_abi::Contract::load(Cursor::new(ABI))
                .expect("failed to load bridge EthereumEventConfigurationContract ABI"),
        );

        Ok(Self {
            transport,
            contract,
        })
    }

    #[inline]
    fn message(&self, addr: MsgAddrStd, name: &str) -> ContractResult<MessageBuilder> {
        MessageBuilder::new(
            Cow::Owned(ContractConfig {
                account: MsgAddressInt::AddrStd(addr),
                timeout_sec: 60,
            }),
            &self.contract,
            self.transport.as_ref(),
            name,
        )
    }

    pub async fn get_details(
        &self,
        addr: MsgAddrStd,
    ) -> ContractResult<EthereumEventConfiguration> {
        self.message(addr, "getDetails")?
            .run_local()
            .await?
            .parse_all()
    }

    pub async fn get_details_hash(&self, addr: MsgAddrStd) -> ContractResult<UInt256> {
        self.message(addr, "getDetails")?.run_local().await?.hash()
    }
}

impl Contract for EthereumEventConfigurationContract {
    #[inline]
    fn abi(&self) -> &Arc<ton_abi::Contract> {
        &self.contract
    }
}

const ABI: &str = include_str!("../../../abi/EthereumEventConfiguration.abi.json");