use cw_orch::{interface, prelude::*};

pub use enterprise_treasury_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use enterprise_treasury::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct EnterpriseTreasuryContract;

impl<Chain: CwEnv> Uploadable for EnterpriseTreasuryContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("enterprise_treasury.wasm")
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