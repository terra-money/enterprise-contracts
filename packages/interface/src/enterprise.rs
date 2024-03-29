use cw_orch::{interface, prelude::*};

use enterprise::contract;
pub use enterprise_protocol::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct EnterpriseContract;

impl<Chain: CwEnv> Uploadable for EnterpriseContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("enterprise")
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
