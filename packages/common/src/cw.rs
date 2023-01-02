use cosmwasm_std::{
    wasm_execute, wasm_instantiate, Addr, Api, CanonicalAddr, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, QuerierWrapper, Response, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_storage_plus::{Bound, PrimaryKey};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub struct Context<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
}

impl<'a> Context<'a> {
    pub fn from(deps: DepsMut<'a>, env: Env, info: MessageInfo) -> Context<'a> {
        Context { deps, env, info }
    }

    pub fn to_query(&'a self) -> QueryContext<'a> {
        QueryContext {
            deps: self.deps.as_ref(),
            env: self.env.clone(),
        }
    }
}

pub struct QueryContext<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}

impl<'a> QueryContext<'a> {
    pub fn from(deps: Deps<'a>, env: Env) -> QueryContext<'a> {
        QueryContext { deps, env }
    }
    pub fn clone(&self) -> QueryContext<'a> {
        QueryContext {
            deps: self.deps.clone(),
            env: self.env.clone(),
        }
    }
}

pub trait ContextWrapper<'a> {
    fn storage(&self) -> &dyn Storage;
    fn api(&self) -> &dyn Api;
    fn querier(&self) -> QuerierWrapper<'a>;
    fn env(&self) -> &Env;
    fn info(&self) -> &MessageInfo;
}

impl<'a> ContextWrapper<'a> for Context<'a> {
    fn storage(&self) -> &dyn Storage {
        self.deps.storage
    }

    fn api(&self) -> &dyn Api {
        self.deps.api
    }

    fn querier(&self) -> QuerierWrapper<'a> {
        self.deps.querier
    }

    fn env(&self) -> &Env {
        &self.env
    }

    fn info(&self) -> &MessageInfo {
        &self.info
    }
}

pub struct RangeArgs<'a, K: PrimaryKey<'a>> {
    pub min: Option<Bound<'a, K>>,
    pub max: Option<Bound<'a, K>>,
    pub order: cosmwasm_std::Order,
}

impl<'a, K: PrimaryKey<'a>> Default for RangeArgs<'a, K> {
    fn default() -> Self {
        RangeArgs {
            min: None,
            max: None,
            order: cosmwasm_std::Order::Ascending,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    #[serde(alias = "asc")]
    Ascending,
    #[serde(alias = "dsc")]
    Descending,
}

impl From<Order> for cosmwasm_std::Order {
    fn from(source: Order) -> Self {
        match source {
            Order::Ascending => cosmwasm_std::Order::Ascending,
            Order::Descending => cosmwasm_std::Order::Descending,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pagination<PK> {
    pub start_after: Option<PK>,
    pub end_at: Option<PK>,
    pub limit: Option<u64>,
    pub order_by: Option<Order>,
}

pub fn send_tokens(
    asset_token: impl Into<String>,
    recipient: impl Into<String>,
    amount: u128,
    method: &str,
) -> StdResult<Response> {
    let recipient = recipient.into();
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(wasm_execute(
            asset_token.into(),
            &Cw20ExecuteMsg::Transfer {
                recipient: recipient.clone(),
                amount: Uint128::new(amount),
            },
            vec![],
        )?))
        .add_attributes(vec![
            ("method", method),
            ("recipient", &recipient),
            ("amount", amount.to_string().as_str()),
        ]))
}

pub trait WasmMsgExt
where
    Self: Sized + Serialize,
{
    fn wasm_instantiate(
        &self,
        code_id: u64,
        funds: Vec<Coin>,
        label: String,
    ) -> StdResult<WasmMsg> {
        wasm_instantiate(code_id, self, funds, label)
    }

    fn wasm_execute(
        &self,
        contract_addr: impl Into<String>,
        funds: Vec<Coin>,
    ) -> StdResult<WasmMsg> {
        wasm_execute(contract_addr, self, funds)
    }
}

pub trait AddrExt {
    fn addr_validate(&self, ctx: &mut Context) -> StdResult<Addr>
    where
        Self: AsRef<str>,
    {
        ctx.deps.api.addr_validate(self.as_ref())
    }

    fn addr_canonicalize(&self, ctx: &mut Context) -> StdResult<CanonicalAddr>
    where
        Self: AsRef<str>,
    {
        ctx.deps.api.addr_canonicalize(self.as_ref())
    }

    fn addr_humanize(&self, ctx: &mut Context) -> StdResult<Addr>
    where
        Self: AsRef<CanonicalAddr>,
    {
        ctx.deps.api.addr_humanize(self.as_ref())
    }
}

impl AddrExt for String {}
impl AddrExt for str {}

pub mod testing {
    use cosmwasm_std::{
        coins, Addr, BlockInfo, Coin, ContractInfo, Deps, DepsMut, Env, MessageInfo, Timestamp,
        TransactionInfo,
    };

    use crate::cw::{Context, QueryContext};

    pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

    /// Returns a mocked Context DI wrapper.
    pub fn mock_ctx(deps: DepsMut) -> Context {
        Context {
            deps,
            env: mock_env(),
            info: mock_info("owner", &coins(2, "token")),
        }
    }

    /// Returns a mocked QueryContext DI wrapper.
    pub fn mock_query_ctx<'a>(deps: Deps<'a>, env: &'a Env) -> QueryContext<'a> {
        QueryContext {
            deps,
            env: env.clone(),
        }
    }

    /// Returns a default environment with height, time, chain_id, and contract address
    /// You can submit as is to most contracts, or modify height/time if you want to
    /// test for expiration.
    ///
    /// This is intended for use in test code only.
    pub fn mock_env() -> Env {
        Env {
            block: BlockInfo {
                height: 12_345,
                time: Timestamp::from_nanos(1_571_797_419_879_305_533),
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            transaction: Some(TransactionInfo { index: 3 }),
            contract: ContractInfo {
                address: Addr::unchecked(MOCK_CONTRACT_ADDR),
            },
        }
    }

    /// Just set sender and funds for the message.
    /// This is intended for use in test code only.
    pub fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
        MessageInfo {
            sender: Addr::unchecked(sender),
            funds: funds.to_vec(),
        }
    }
}
