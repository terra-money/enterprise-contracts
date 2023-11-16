#[cfg(feature = "interface")]
pub mod attestation;
#[cfg(feature = "interface")]
pub mod denom_staking_membership;
#[cfg(feature = "interface")]
pub mod enterprise;
#[cfg(feature = "interface")]
pub mod enterprise_facade;
#[cfg(feature = "interface")]
pub mod enterprise_facade_v1;
#[cfg(feature = "interface")]
pub mod enterprise_facade_v2;
#[cfg(feature = "interface")]
pub mod enterprise_factory;
#[cfg(feature = "interface")]
pub mod enterprise_governance;
#[cfg(feature = "interface")]
pub mod enterprise_governance_controller;
#[cfg(feature = "interface")]
pub mod enterprise_outposts;
#[cfg(feature = "interface")]
pub mod enterprise_treasury;
#[cfg(feature = "interface")]
pub mod enterprise_versioning;
#[cfg(feature = "interface")]
pub mod funds_distributor;
#[cfg(feature = "interface")]
pub mod multisig_membership;
#[cfg(feature = "interface")]
pub mod nft_staking_membership;
#[cfg(feature = "interface")]
pub mod token_staking_membership;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
