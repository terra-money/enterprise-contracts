use crate::api::{AttestationTextResponse, HasUserSignedParams, HasUserSignedResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub attestation_text: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SignAttestation {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AttestationTextResponse)]
    AttestationText {},
    #[returns(HasUserSignedResponse)]
    HasUserSigned(HasUserSignedParams),
}

#[cw_serde]
pub struct MigrateMsg {}
