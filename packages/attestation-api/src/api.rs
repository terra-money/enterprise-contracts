use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct HasUserSignedParams {
    pub user: String,
}

////// Responses

#[cw_serde]
pub struct AttestationTextResponse {
    pub text: String,
}

#[cw_serde]
pub struct HasUserSignedResponse {
    pub has_signed: bool,
}
