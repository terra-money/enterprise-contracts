use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Timestamp, Uint64};
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

    pub fn branch(&mut self) -> Context {
        Context {
            deps: self.deps.branch(),
            env: self.env.clone(),
            info: self.info.clone(),
        }
    }
}

#[derive(Clone)]
pub struct QueryContext<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}

impl<'a> QueryContext<'a> {
    pub fn from(deps: Deps<'a>, env: Env) -> QueryContext<'a> {
        QueryContext { deps, env }
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
pub enum ReleaseAt {
    Timestamp(Timestamp),
    Height(Uint64),
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
