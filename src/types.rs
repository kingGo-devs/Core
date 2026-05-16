use soroban_sdk::{contracttype, Address, Bytes, Map};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalState {
    Active,
    Passed,
    Rejected,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub state: ProposalState,
    pub description_hash: Bytes,
    pub total_yes_votes: i128,
    pub total_no_votes: i128,
    pub expiration_ledger: u32,
    pub creator: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    ProposalCount,
    Proposal(u32),
    VoterRegistry(u32),
    Admin,
    TokenAddress,
    MinThreshold,
}
