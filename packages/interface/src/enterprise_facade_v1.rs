use cw_orch::{interface, prelude::*};

pub use enterprise_facade_api::msg::{ExecuteMsg, QueryMsg};
use enterprise_facade_v1::contract;
pub use enterprise_facade_v1::msg::InstantiateMsg;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct EnterpriseFacadeV1Contract;

impl<Chain: CwEnv> Uploadable for EnterpriseFacadeV1Contract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("enterprise_facade_v1")
            .unwrap()
    }
    // Return a CosmWasm contract wrapper
    fn wrapper(&self) -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            contract::execute,
            contract::instantiate,
            contract::query,
        ))
    }
}
