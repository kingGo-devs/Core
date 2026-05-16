#![no_std]

mod contract;
mod events;
mod types;

pub use contract::{VotingContract, VotingContractClient};
pub use types::{DataKey, Proposal, ProposalState};

#[cfg(test)]
mod tests;
