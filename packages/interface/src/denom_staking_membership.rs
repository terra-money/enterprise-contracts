use cw_orch::{interface, prelude::*};

pub use denom_staking_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use denom_staking_membership::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct DenomStakingMembershipContract;

impl<Chain: CwEnv> Uploadable for DenomStakingMembershipContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("denom_staking_membership")
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
