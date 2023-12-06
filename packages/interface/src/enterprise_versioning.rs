use cw_orch::{interface, prelude::*};

use enterprise_versioning::contract;
pub use enterprise_versioning_api::api::{
    AddVersionMsg, EditVersionMsg, Version, VersionInfo, VersionParams, VersionsParams,
};
pub use enterprise_versioning_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct EnterpriseVersioningContract;

impl<Chain: CwEnv> Uploadable for EnterpriseVersioningContract<Chain> {
    // Return the path to the wasm file
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("enterprise_versioning")
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
