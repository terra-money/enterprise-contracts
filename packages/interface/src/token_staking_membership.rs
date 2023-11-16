use cw_orch::{interface, prelude::*};

pub use token_staking_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use token_staking_membership::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct TokenStakingMembershipContract;

impl<Chain: CwEnv> Uploadable for TokenStakingMembershipContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("token_staking_membership_contract.wasm")
            .unwrap()
    }
    // Return a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                contract::execute,
                contract::instantiate,
                contract::query,
            )
                .with_migrate(contract::migrate),
        )
    }
}