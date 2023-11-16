use cw_orch::{interface, prelude::*};

pub use nft_staking_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use nft_staking_membership::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct NftStakingMembershipContract;

impl<Chain: CwEnv> Uploadable for NftStakingMembershipContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("nft_staking_membership.wasm")
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