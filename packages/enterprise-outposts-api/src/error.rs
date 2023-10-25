use cosmwasm_std::StdError;
use cw_utils::ParseReplyError;
use thiserror::Error;

pub type EnterpriseOutpostsResult<T> = Result<T, EnterpriseOutpostsError>;

#[derive(Error, Debug, PartialEq)]
pub enum EnterpriseOutpostsError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Trying to add a proxy for a chain ID that already has a DAO-owned proxy deployed")]
    ProxyAlreadyExistsForChainId,

    #[error("Trying to add a treasury for a chain ID that already has a treasury deployed")]
    TreasuryAlreadyExistsForChainId,
}

impl EnterpriseOutpostsError {
    /// Converts this EnterpriseOutpostsError into a StdError.
    pub fn std_err(&self) -> StdError {
        StdError::generic_err(format!("{:?}", self))
    }
}

impl From<serde_json_wasm::ser::Error> for EnterpriseOutpostsError {
    fn from(value: serde_json_wasm::ser::Error) -> Self {
        EnterpriseOutpostsError::Std(StdError::generic_err(value.to_string()))
    }
}

impl From<ParseReplyError> for EnterpriseOutpostsError {
    fn from(value: ParseReplyError) -> Self {
        EnterpriseOutpostsError::Std(StdError::generic_err(value.to_string()))
    }
}

impl From<bech32_no_std::Error> for EnterpriseOutpostsError {
    fn from(value: bech32_no_std::Error) -> Self {
        EnterpriseOutpostsError::Std(StdError::generic_err(value.to_string()))
    }
}
