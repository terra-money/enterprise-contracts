use cw_orch::{interface, prelude::*};

pub use funds_distributor_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use funds_distributor::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct FundsDistributorContract;

impl<Chain: CwEnv> Uploadable for FundsDistributorContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("funds_distributor.wasm")
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