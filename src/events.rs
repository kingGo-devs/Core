use soroban_sdk::{Address, Env, Symbol};

const TOPIC_PROPOSAL_CREATED: &str = "proposal_created";
const TOPIC_VOTE_CAST: &str = "vote_cast";

pub fn emit_proposal_created(env: &Env, proposal_id: u32, creator: &Address) {
    env.events().publish(
        (Symbol::new(env, TOPIC_PROPOSAL_CREATED),),
        (proposal_id, creator.clone()),
    );
}

pub fn emit_vote_cast(
    env: &Env,
    voter: &Address,
    proposal_id: u32,
    support: bool,
    voting_power: i128,
) {
    env.events().publish(
        (Symbol::new(env, TOPIC_VOTE_CAST),),
        (voter.clone(), proposal_id, support, voting_power),
    );
}
