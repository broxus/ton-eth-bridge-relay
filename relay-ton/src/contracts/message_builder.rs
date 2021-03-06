use ton_abi::{Contract, Function, Token, TokenValue};
use ton_block::MsgAddress;

use super::errors::*;
use super::prelude::*;
use crate::models::*;
use crate::prelude::*;
use crate::transport::*;

impl From<ExternalMessageHeader> for HashMap<String, TokenValue> {
    fn from(header: ExternalMessageHeader) -> Self {
        let mut result = HashMap::with_capacity(2);
        result.insert("time".to_string(), TokenValue::Time(header.time));
        result.insert("expire".to_string(), TokenValue::Expire(header.expire));
        result.insert("pubkey".to_string(), TokenValue::PublicKey(header.pubkey));
        result
    }
}

pub fn make_external_header(timeout_sec: u32, keypair: Option<&Keypair>) -> ExternalMessageHeader {
    let time = Utc::now().timestamp_millis() as u64;
    let expire = ((time / 1000) + timeout_sec as u64) as u32;
    ExternalMessageHeader {
        time,
        expire,
        pubkey: keypair.map(|pair| pair.public),
    }
}

pub struct MessageBuilder<'a>(MessageBuilderImpl<'a, dyn Transport>);

impl<'a> MessageBuilder<'a> {
    pub fn new(
        config: Cow<'a, ContractConfig>,
        contract: &'a Contract,
        transport: &'a dyn Transport,
        name: &str,
    ) -> ContractResult<Self> {
        Ok(Self(MessageBuilderImpl::new(
            config, contract, transport, name,
        )?))
    }

    #[allow(dead_code)]
    pub fn arg<A>(self, value: A) -> Self
    where
        A: FunctionArg,
    {
        Self(self.0.arg(value))
    }

    #[allow(dead_code)]
    pub fn args<A>(self, values: A) -> Self
    where
        A: FunctionArgsGroup,
    {
        Self(self.0.args(values))
    }

    #[allow(dead_code)]
    pub fn mark_local(self) -> Self {
        Self(self.0.mark_local())
    }

    pub fn build(self, keypair: Option<&Keypair>) -> ContractResult<ExternalMessage> {
        self.0.build(keypair)
    }

    pub fn build_internal(self, src: MsgAddressInt, value: u64) -> ContractResult<InternalMessage> {
        self.0.build_internal(src, value)
    }

    pub fn build_internal_body(self) -> ContractResult<BuilderData> {
        self.0.build_internal_body()
    }

    pub async fn run_local(self) -> ContractResult<ContractOutput> {
        let transport = self.0.transport;
        let output = transport
            .run_local(self.0.function, self.build(None)?)
            .await?;
        Ok(output)
    }
}

pub struct SignedMessageBuilder<'a>(&'a Keypair, MessageBuilderImpl<'a, dyn AccountSubscription>);

impl<'a> SignedMessageBuilder<'a> {
    pub fn new(
        config: Cow<'a, ContractConfig>,
        contract: &'a Contract,
        transport: &'a dyn AccountSubscription,
        keypair: &'a Keypair,
        name: &str,
    ) -> ContractResult<Self> {
        Ok(Self(
            keypair,
            MessageBuilderImpl::new(config, contract, transport, name)?,
        ))
    }

    #[allow(dead_code)]
    pub fn arg<A>(self, value: A) -> Self
    where
        A: FunctionArg,
    {
        Self(self.0, self.1.arg(value))
    }

    #[allow(dead_code)]
    pub fn args<A>(self, values: A) -> Self
    where
        A: FunctionArgsGroup,
    {
        Self(self.0, self.1.args(values))
    }

    #[allow(dead_code)]
    pub fn mark_local(self) -> Self {
        Self(self.0, self.1.mark_local())
    }

    pub fn build(self, with_signature: bool) -> ContractResult<ExternalMessage> {
        self.1
            .build(if with_signature { Some(self.0) } else { None })
    }

    #[allow(dead_code)]
    pub async fn run_local(self) -> ContractResult<ContractOutput> {
        let transport = self.1.transport;
        let output = transport
            .run_local(self.1.function, self.build(false)?)
            .await?;
        Ok(output)
    }

    pub async fn send(self) -> ContractResult<ContractOutput> {
        let function = Arc::new(self.1.function.clone());
        let output = self
            .1
            .transport
            .send_message(function, self.build(true)?)
            .await?;
        Ok(output)
    }
}

struct MessageBuilderImpl<'a, T: ?Sized> {
    config: Cow<'a, ContractConfig>,
    function: &'a Function,
    transport: &'a T,
    input: Vec<Token>,
    run_local: bool,
}

impl<'a, T> MessageBuilderImpl<'a, T>
where
    T: ?Sized,
{
    pub fn new(
        config: Cow<'a, ContractConfig>,
        contract: &'a Contract,
        transport: &'a T,
        name: &str,
    ) -> ContractResult<Self> {
        let function = contract
            .function(name)
            .map_err(|_| ContractError::InvalidAbi)?;
        let input = Vec::with_capacity(function.inputs.len());

        Ok(Self {
            config,
            function,
            transport,
            input,
            run_local: false,
        })
    }

    pub fn arg<A>(mut self, value: A) -> Self
    where
        A: FunctionArg,
    {
        let name = &self.function.inputs[self.input.len()].name;
        self.input.push(Token::new(name, value.token_value()));
        self
    }

    pub fn args<A>(mut self, values: A) -> Self
    where
        A: FunctionArgsGroup,
    {
        let token_values = values.token_values();
        let args_from = self.input.len();
        let args_to = args_from + token_values.len();

        let inputs = &self.function.inputs;
        self.input.extend(
            (args_from..args_to)
                .into_iter()
                .map(|i| inputs[i].name.as_ref())
                .zip(token_values.into_iter())
                .map(|(name, value)| Token::new(name, value)),
        );
        self
    }

    #[allow(dead_code)]
    pub fn mark_local(mut self) -> Self {
        self.run_local = true;
        self
    }

    pub fn build(self, keypair: Option<&Keypair>) -> ContractResult<ExternalMessage> {
        let header = make_external_header(self.config.timeout_sec, keypair);
        let encoded_input = self
            .function
            .encode_input(&header.clone().into(), &self.input, false, keypair)
            .map_err(|_| ContractError::InvalidInput)?;

        Ok(ExternalMessage {
            dest: self.config.account.clone(),
            init: None,
            body: Some(encoded_input.into()),
            header,
            run_local: self.run_local,
        })
    }

    pub fn build_internal(self, src: MsgAddressInt, value: u64) -> ContractResult<InternalMessage> {
        let header = InternalMessageHeader { src, value };
        let body = self.build_internal_body()?;

        Ok(InternalMessage {
            dest: self.config.account.clone(),
            init: None,
            body: Some(body.into()),
            header,
        })
    }

    pub fn build_internal_body(&self) -> ContractResult<BuilderData> {
        self.function
            .encode_input(&Default::default(), &self.input, true, None)
            .map_err(|_| ContractError::InvalidInput)
    }
}

impl FunctionArg for bool {
    fn token_value(self) -> TokenValue {
        TokenValue::Bool(self)
    }
}

impl FunctionArg for &str {
    fn token_value(self) -> TokenValue {
        TokenValue::Bytes(self.as_bytes().into())
    }
}

impl FunctionArg for Vec<u8> {
    fn token_value(self) -> TokenValue {
        TokenValue::Bytes(self)
    }
}

impl FunctionArg for AccountId {
    fn token_value(self) -> TokenValue {
        TokenValue::Address(MsgAddress::AddrStd(self.into()))
    }
}

impl FunctionArg for MsgAddrStd {
    fn token_value(self) -> TokenValue {
        TokenValue::Address(MsgAddress::AddrStd(self))
    }
}

impl FunctionArg for MsgAddressInt {
    fn token_value(self) -> TokenValue {
        TokenValue::Address(match self {
            MsgAddressInt::AddrStd(addr) => MsgAddress::AddrStd(addr),
            MsgAddressInt::AddrVar(addr) => MsgAddress::AddrVar(addr),
        })
    }
}

impl FunctionArg for EthAddress {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: num_bigint::BigUint::from_bytes_be(self.as_bytes()),
            size: 160,
        })
    }
}

impl FunctionArg for primitive_types::H256 {
    fn token_value(self) -> TokenValue {
        BigUint256(num_bigint::BigUint::from_bytes_be(self.as_bytes())).token_value()
    }
}

impl FunctionArg for UInt256 {
    fn token_value(self) -> TokenValue {
        BigUint256(num_bigint::BigUint::from_bytes_be(self.as_slice())).token_value()
    }
}

impl FunctionArg for UInt128 {
    fn token_value(self) -> TokenValue {
        BigUint128(num_bigint::BigUint::from_bytes_be(self.as_slice())).token_value()
    }
}

pub struct BigUint256(pub BigUint);

impl FunctionArg for BigUint256 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: self.0,
            size: 256,
        })
    }
}

pub struct BigUint128(pub BigUint);

impl FunctionArg for BigUint128 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: self.0,
            size: 128,
        })
    }
}

impl FunctionArg for i8 {
    fn token_value(self) -> TokenValue {
        TokenValue::Int(ton_abi::Int {
            number: BigInt::from(self),
            size: 8,
        })
    }
}

impl FunctionArg for u8 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 8,
        })
    }
}

impl FunctionArg for u16 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 16,
        })
    }
}

impl FunctionArg for u32 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 32,
        })
    }
}

impl FunctionArg for u64 {
    fn token_value(self) -> TokenValue {
        TokenValue::Uint(ton_abi::Uint {
            number: BigUint::from(self),
            size: 64,
        })
    }
}

impl FunctionArg for BuilderData {
    fn token_value(self) -> TokenValue {
        TokenValue::Cell(self.into())
    }
}

impl FunctionArg for ton_types::Cell {
    fn token_value(self) -> TokenValue {
        TokenValue::Cell(self)
    }
}

impl<T> FunctionArg for Vec<T>
where
    T: StandaloneToken + FunctionArg,
{
    fn token_value(self) -> TokenValue {
        TokenValue::Array(self.into_iter().map(FunctionArg::token_value).collect())
    }
}

impl FunctionArg for TokenValue {
    fn token_value(self) -> TokenValue {
        self
    }
}

impl<T> FunctionArg for &T
where
    T: Clone + FunctionArg,
{
    fn token_value(self) -> TokenValue {
        self.clone().token_value()
    }
}

impl<T> FunctionArgsGroup for &T
where
    T: Clone + FunctionArgsGroup,
{
    fn token_values(self) -> Vec<TokenValue> {
        self.clone().token_values()
    }
}

pub trait FunctionArg {
    fn token_value(self) -> TokenValue;
}

pub trait FunctionArgsGroup {
    fn token_values(self) -> Vec<TokenValue>;
}
