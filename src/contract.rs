use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, Map};

use crate::events;
use crate::types::*;

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    pub fn initialize(env: Env, admin: Address, token_address: Address, min_threshold: i128) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::TokenAddress, &token_address);
        env.storage()
            .instance()
            .set(&DataKey::MinThreshold, &min_threshold);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &0u32);
    }

    pub fn initialize_proposal(
        env: Env,
        from: Address,
        description_hash: Bytes,
        expiration_ledger: u32,
    ) -> u32 {
        from.require_auth();

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();
        let min_threshold: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinThreshold)
            .unwrap();

        let token_client = token::Client::new(&env, &token_address);
        let balance = token_client.balance(&from);

        if balance < min_threshold {
            panic!("insufficient token balance to create proposal");
        }

        let mut count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap();
        count += 1;

        let proposal = Proposal {
            state: ProposalState::Active,
            description_hash,
            total_yes_votes: 0,
            total_no_votes: 0,
            expiration_ledger,
            creator: from.clone(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Proposal(count), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &count);
        env.storage()
            .instance()
            .set(&DataKey::VoterRegistry(count), &Map::new(&env));

        events::emit_proposal_created(&env, count, &from);

        count
    }

    pub fn vote(env: Env, voter: Address, proposal_id: u32, support: bool) {
        voter.require_auth();

        let proposal_key = DataKey::Proposal(proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&proposal_key)
            .unwrap();

        if !matches!(proposal.state, ProposalState::Active) {
            panic!("proposal is not active");
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger >= proposal.expiration_ledger {
            proposal.state = ProposalState::Expired;
            env.storage()
                .instance()
                .set(&proposal_key, &proposal);
            panic!("proposal has expired");
        }

        let voter_registry_key = DataKey::VoterRegistry(proposal_id);
        let mut voters: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&voter_registry_key)
            .unwrap();

        if voters.contains_key(voter.clone()) {
            panic!("voter has already voted on this proposal");
        }

        let token_address: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenAddress)
            .unwrap();

        let token_client = token::Client::new(&env, &token_address);
        let balance = token_client.balance(&voter);

        if balance <= 0 {
            panic!("voter has no voting power");
        }

        if support {
            proposal.total_yes_votes += balance;
        } else {
            proposal.total_no_votes += balance;
        }

        voters.set(voter.clone(), true);

        env.storage()
            .instance()
            .set(&proposal_key, &proposal);
        env.storage()
            .instance()
            .set(&voter_registry_key, &voters);

        events::emit_vote_cast(&env, &voter, proposal_id, support, balance);
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Proposal {
        env.storage()
            .instance()
            .get(&DataKey::Proposal(proposal_id))
            .unwrap()
    }

    pub fn get_proposal_count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap()
    }

    pub fn has_voted(env: Env, voter: Address, proposal_id: u32) -> bool {
        let voters: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&DataKey::VoterRegistry(proposal_id))
            .unwrap();
        voters.contains_key(voter)
    }

    pub fn get_voter_registry(env: Env, proposal_id: u32) -> Map<Address, bool> {
        env.storage()
            .instance()
            .get(&DataKey::VoterRegistry(proposal_id))
            .unwrap()
    }
}
