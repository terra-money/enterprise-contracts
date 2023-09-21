use crate::state::ProposalExecutabilityStatus::{Draw, NotExecutable, Passed, Rejected};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use enterprise_governance_controller_api::api::ProposalInfo;
use enterprise_governance_controller_api::api::{CouncilGovConfig, GovConfig, ProposalId};
use poll_engine_api::api::{PollRejectionReason, PollStatus};
use PollRejectionReason::{
    IsRejectingOutcome, IsVetoOutcome, OutcomeDraw, QuorumAndThresholdNotReached, QuorumNotReached,
    ThresholdNotReached,
};

#[cw_serde]
pub struct State {
    pub proposal_being_created: Option<ProposalInfo>,
    pub proposal_being_executed: Option<ProposalId>,
    pub proposal_being_voted_on: Option<ProposalBeingVotedOn>,
}

#[cw_serde]
pub struct ProposalBeingVotedOn {
    pub proposal_id: ProposalId,
    pub executability_status: ProposalExecutabilityStatus,
}

#[cw_serde]
pub enum ProposalExecutabilityStatus {
    /// Conditions such as quorum or threshold not met - cannot execute
    NotExecutable,
    Passed {
        outcome: u8,
    },
    Rejected {
        /// Whether the rejection was a veto or a normal rejection
        veto: bool,
    },
    Draw {
        outcome1: u8,
        outcome2: u8,
        votes_for_each: Uint128,
    },
}

impl From<PollStatus> for ProposalExecutabilityStatus {
    fn from(poll_status: PollStatus) -> Self {
        match poll_status {
            PollStatus::InProgress { .. } => NotExecutable,
            PollStatus::Passed { outcome, .. } => Passed { outcome },
            PollStatus::Rejected { reason } => match reason {
                QuorumNotReached | ThresholdNotReached | QuorumAndThresholdNotReached => {
                    NotExecutable
                }
                IsRejectingOutcome => Rejected { veto: false },
                IsVetoOutcome => Rejected { veto: true },
                OutcomeDraw(outcome1, outcome2, votes_for_each) => Draw {
                    outcome1,
                    outcome2,
                    votes_for_each,
                },
            },
        }
    }
}

pub const STATE: Item<State> = Item::new("state");

pub const ENTERPRISE_CONTRACT: Item<Addr> = Item::new("enterprise_contract");

pub const GOV_CONFIG: Item<GovConfig> = Item::new("gov_config");

pub const COUNCIL_GOV_CONFIG: Item<Option<CouncilGovConfig>> = Item::new("council_gov_config");
