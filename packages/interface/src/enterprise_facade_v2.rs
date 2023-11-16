use cw_orch::{interface, prelude::*};

pub use enterprise_facade_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use enterprise_facade_v2::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct EnterpriseFacadeContract;

impl<Chain: CwEnv> Uploadable for EnterpriseFacadeContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("enterprise_facade_v2.wasm")
            .unwrap()
    }
    // Return a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                contract::execute,
                contract::instantiate,
                contract::query,
            ),
        )
    }
}