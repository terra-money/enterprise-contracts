use cw_orch::{interface, prelude::*};

pub use enterprise_outposts_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use enterprise_outposts::contract;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct EnterpriseOutpostsContract;

impl<Chain: CwEnv> Uploadable for EnterpriseOutpostsContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("enterprise_outposts.wasm")
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