#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, log, symbol_short, Address, Env, Map,
    String, Symbol, Vec,
};
use stellarmarket_shared::{MarketStatus, SharedError};

// ============================================================
// STORAGE KEYS
// ============================================================

const KEY_MARKET_COUNT: Symbol = symbol_short!("MKT_CNT");
const KEY_PROPOSAL_COUNT: Symbol = symbol_short!("PROP_CNT");
const KEY_MAINTAINERS: Symbol = symbol_short!("MNTNERS");
const KEY_THRESHOLD: Symbol = symbol_short!("THRESHOLD");
const KEY_MARKETS: Symbol = symbol_short!("MARKETS");
const KEY_PROPOSALS: Symbol = symbol_short!("PROPOSALS");

// ============================================================
// DATA TYPES
// ============================================================

/// Metadata stored for every approved prediction market.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MarketMetadata {
    /// Unique market identifier (auto-incremented).
    pub market_id: u64,
    /// The prediction question, e.g. "Will X win the election?".
    pub question: String,
    /// Possible outcome labels, e.g. ["YES", "NO"].
    pub outcomes: Vec<String>,
    /// Unix timestamp after which trading closes and resolution begins.
    pub resolution_date: u64,
    /// Identifier of the oracle contract or provider responsible for resolution.
    pub oracle_id: Address,
    /// Current lifecycle status of the market.
    pub status: MarketStatus,
    /// Stellar address that proposed this market.
    pub proposer: Address,
    /// Ledger timestamp when the market was approved.
    pub approved_at: u64,
}

/// An unreviewed market proposal submitted by a user.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MarketProposal {
    /// Unique proposal identifier (auto-incremented).
    pub proposal_id: u64,
    /// The prediction question.
    pub question: String,
    /// Possible outcome labels.
    pub outcomes: Vec<String>,
    /// Requested resolution date (Unix timestamp).
    pub resolution_date: u64,
    /// Requested oracle address.
    pub oracle_id: Address,
    /// Address that submitted the proposal.
    pub proposer: Address,
    /// Ledger timestamp of submission.
    pub submitted_at: u64,
    /// Number of maintainer approvals received so far.
    pub approval_count: u32,
    /// Whether this proposal has been decided (approved or rejected).
    pub decided: bool,
}

// ============================================================
// ERRORS
// ============================================================

/// Error types returned by the MarketFactory contract.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// Contract is not initialized yet.
    NotInitialized = 1,
    /// Contract has already been initialized.
    AlreadyInitialized = 2,
    /// Caller is not a registered maintainer.
    NotAMaintainer = 3,
    /// The requested proposal ID does not exist.
    ProposalNotFound = 4,
    /// The proposal has already been decided (approved or rejected).
    ProposalAlreadyDecided = 5,
    /// This maintainer has already approved this proposal.
    AlreadyApprovedByMaintainer = 6,
    /// The proposed question is invalid (empty).
    InvalidQuestion = 7,
    /// The proposed outcomes are invalid (need at least 2 outcomes).
    InvalidOutcomes = 8,
    /// The resolution date is invalid (must be in the future).
    InvalidResolutionDate = 9,
    /// The approval threshold is invalid.
    InvalidThreshold = 10,
    /// The requested market ID does not exist.
    MarketNotFound = 11,
}

// ============================================================
// CONTRACT IMPLEMENTATION
// ============================================================

/// The MarketFactory contract, serving as the registry for prediction markets.
#[contract]
pub struct MarketFactory;

#[contractimpl]
impl MarketFactory {
    /// Initializes the MarketFactory contract with maintainers and approval threshold.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `maintainers` - Vector of registered maintainer addresses.
    /// * `approval_threshold` - Number of maintainer approvals required to accept a proposal.
    pub fn initialize(
        env: Env,
        maintainers: Vec<Address>,
        approval_threshold: u32,
    ) -> Result<(), Error> {
        if env.storage().persistent().has(&KEY_MAINTAINERS) {
            return Err(Error::AlreadyInitialized);
        }

        if approval_threshold == 0 || approval_threshold > maintainers.len() {
            return Err(Error::InvalidThreshold);
        }

        env.storage()
            .persistent()
            .set(&KEY_MAINTAINERS, &maintainers);
        env.storage()
            .persistent()
            .set(&KEY_THRESHOLD, &approval_threshold);
        env.storage().persistent().set(&KEY_MARKET_COUNT, &0u64);
        env.storage().persistent().set(&KEY_PROPOSAL_COUNT, &0u64);

        let markets: Map<u64, MarketMetadata> = Map::new(&env);
        let proposals: Map<u64, MarketProposal> = Map::new(&env);
        env.storage().persistent().set(&KEY_MARKETS, &markets);
        env.storage().persistent().set(&KEY_PROPOSALS, &proposals);

        Ok(())
    }

    /// Submits a new market proposal. Any user can call this by providing the required details.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `proposer` - The address of the user submitting the proposal.
    /// * `question` - The prediction question.
    /// * `outcomes` - List of possible outcome labels.
    /// * `resolution_date` - Unix timestamp when trading closes and resolution begins.
    /// * `oracle_id` - The address of the oracle.
    pub fn propose_market(
        env: Env,
        proposer: Address,
        question: String,
        outcomes: Vec<String>,
        resolution_date: u64,
        oracle_id: Address,
    ) -> Result<u64, Error> {
        proposer.require_auth();

        if question.len() == 0 {
            return Err(Error::InvalidQuestion);
        }
        if outcomes.len() < 2 {
            return Err(Error::InvalidOutcomes);
        }
        if resolution_date <= env.ledger().timestamp() {
            return Err(Error::InvalidResolutionDate);
        }

        let mut proposals: Map<u64, MarketProposal> = env
            .storage()
            .persistent()
            .get(&KEY_PROPOSALS)
            .unwrap_or_else(|| Map::new(&env));

        let proposal_count: u64 = env
            .storage()
            .persistent()
            .get(&KEY_PROPOSAL_COUNT)
            .unwrap_or(0);

        let new_proposal_id = proposal_count + 1;

        let proposal = MarketProposal {
            proposal_id: new_proposal_id,
            question: question.clone(),
            outcomes,
            resolution_date,
            oracle_id,
            proposer: proposer.clone(),
            submitted_at: env.ledger().timestamp(),
            approval_count: 0,
            decided: false,
        };

        proposals.set(new_proposal_id, proposal);
        env.storage().persistent().set(&KEY_PROPOSALS, &proposals);
        env.storage()
            .persistent()
            .set(&KEY_PROPOSAL_COUNT, &new_proposal_id);

        env.events().publish(
            (
                Symbol::new(&env, "MarketProposed"),
                new_proposal_id,
                proposer,
            ),
            question,
        );

        Ok(new_proposal_id)
    }

    /// Approves a pending market proposal. Checks caller is a registered maintainer.
    /// If the approval count reaches the threshold, the market is created.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `maintainer` - The address of the maintainer approving the proposal.
    /// * `proposal_id` - The ID of the proposal to approve.
    pub fn approve_market(env: Env, maintainer: Address, proposal_id: u64) -> Result<(), Error> {
        maintainer.require_auth();

        if !env.storage().persistent().has(&KEY_MAINTAINERS) {
            return Err(Error::NotInitialized);
        }

        let maintainers: Vec<Address> = env.storage().persistent().get(&KEY_MAINTAINERS).unwrap();

        if !maintainers.contains(&maintainer) {
            return Err(Error::NotAMaintainer);
        }

        let mut proposals: Map<u64, MarketProposal> = env
            .storage()
            .persistent()
            .get(&KEY_PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.decided {
            return Err(Error::ProposalAlreadyDecided);
        }

        let mut approved_keys = env
            .storage()
            .persistent()
            .get(&symbol_short!("APP_KEYS"))
            .unwrap_or_else(|| Map::new(&env));

        let key = (proposal_id, maintainer.clone());
        if approved_keys.contains_key(key.clone()) {
            return Err(Error::AlreadyApprovedByMaintainer);
        }

        approved_keys.set(key, true);
        env.storage()
            .persistent()
            .set(&symbol_short!("APP_KEYS"), &approved_keys);

        proposal.approval_count += 1;

        let threshold: u32 = env.storage().persistent().get(&KEY_THRESHOLD).unwrap();

        if proposal.approval_count >= threshold {
            proposal.decided = true;

            let mut markets: Map<u64, MarketMetadata> = env
                .storage()
                .persistent()
                .get(&KEY_MARKETS)
                .unwrap_or_else(|| Map::new(&env));

            let market_count: u64 = env
                .storage()
                .persistent()
                .get(&KEY_MARKET_COUNT)
                .unwrap_or(0);

            let new_market_id = market_count + 1;

            let market = MarketMetadata {
                market_id: new_market_id,
                question: proposal.question.clone(),
                outcomes: proposal.outcomes.clone(),
                resolution_date: proposal.resolution_date,
                oracle_id: proposal.oracle_id.clone(),
                status: MarketStatus::Active,
                proposer: proposal.proposer.clone(),
                approved_at: env.ledger().timestamp(),
            };

            markets.set(new_market_id, market);
            env.storage().persistent().set(&KEY_MARKETS, &markets);
            env.storage()
                .persistent()
                .set(&KEY_MARKET_COUNT, &new_market_id);

            env.events().publish(
                (
                    Symbol::new(&env, "MarketApproved"),
                    new_market_id,
                    proposal_id,
                ),
                (),
            );
        }

        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&KEY_PROPOSALS, &proposals);

        Ok(())
    }

    /// Rejects a pending market proposal. Checks caller is a registered maintainer.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `maintainer` - The address of the maintainer rejecting the proposal.
    /// * `proposal_id` - The ID of the proposal to reject.
    /// * `reason` - The rationale for rejection.
    pub fn reject_market(
        env: Env,
        maintainer: Address,
        proposal_id: u64,
        reason: String,
    ) -> Result<(), Error> {
        maintainer.require_auth();

        if !env.storage().persistent().has(&KEY_MAINTAINERS) {
            return Err(Error::NotInitialized);
        }

        let maintainers: Vec<Address> = env.storage().persistent().get(&KEY_MAINTAINERS).unwrap();

        if !maintainers.contains(&maintainer) {
            return Err(Error::NotAMaintainer);
        }

        let mut proposals: Map<u64, MarketProposal> = env
            .storage()
            .persistent()
            .get(&KEY_PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        let mut proposal = proposals.get(proposal_id).ok_or(Error::ProposalNotFound)?;

        if proposal.decided {
            return Err(Error::ProposalAlreadyDecided);
        }

        proposal.decided = true;
        proposals.set(proposal_id, proposal);
        env.storage().persistent().set(&KEY_PROPOSALS, &proposals);

        env.events()
            .publish((Symbol::new(&env, "MarketRejected"), proposal_id), reason);

        Ok(())
    }

    /// Returns the metadata for a specific market ID.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `id` - The ID of the market.
    pub fn get_market(env: Env, id: u64) -> Result<MarketMetadata, Error> {
        let markets: Map<u64, MarketMetadata> = env
            .storage()
            .persistent()
            .get(&KEY_MARKETS)
            .ok_or(Error::MarketNotFound)?;

        markets.get(id).ok_or(Error::MarketNotFound)
    }

    /// Returns the proposal details for a specific proposal ID.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `id` - The ID of the proposal.
    pub fn get_proposal(env: Env, id: u64) -> Result<MarketProposal, Error> {
        let proposals: Map<u64, MarketProposal> = env
            .storage()
            .persistent()
            .get(&KEY_PROPOSALS)
            .ok_or(Error::ProposalNotFound)?;

        proposals.get(id).ok_or(Error::ProposalNotFound)
    }

    /// Returns paginated markets matching status.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `status` - The status to filter by.
    /// * `page` - The zero-indexed page number (10 items per page).
    pub fn list_markets(
        env: Env,
        status: MarketStatus,
        page: u32,
    ) -> Result<Vec<MarketMetadata>, Error> {
        let page_size = 10;
        let mut result = Vec::new(&env);
        let mut skip = page * page_size;
        let mut count = 0;

        let market_count: u64 = env
            .storage()
            .persistent()
            .get(&KEY_MARKET_COUNT)
            .unwrap_or(0);

        let markets: Map<u64, MarketMetadata> = env
            .storage()
            .persistent()
            .get(&KEY_MARKETS)
            .unwrap_or_else(|| Map::new(&env));

        for i in 1..=market_count {
            if let Some(market) = markets.get(i) {
                if market.status == status {
                    if skip > 0 {
                        skip -= 1;
                    } else {
                        result.push_back(market);
                        count += 1;
                        if count >= page_size {
                            break;
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String, Vec};

    #[test]
    fn test_initialize_once() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MarketFactory);
        let client = MarketFactoryClient::new(&env, &contract_id);

        let m1 = Address::generate(&env);
        let m2 = Address::generate(&env);
        let maintainers = Vec::from_array(&env, [m1.clone(), m2.clone()]);

        client.initialize(&maintainers, &2);

        // Trying to initialize again should result in error
        let res = client.try_initialize(&maintainers, &2);
        assert!(res.is_err());
    }

    #[test]
    fn test_propose_market_and_approval_flow() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MarketFactory);
        let client = MarketFactoryClient::new(&env, &contract_id);

        let m1 = Address::generate(&env);
        let m2 = Address::generate(&env);
        let maintainers = Vec::from_array(&env, [m1.clone(), m2.clone()]);

        client.initialize(&maintainers, &2);

        let proposer = Address::generate(&env);
        let question = String::from_str(&env, "Will Bitcoin exceed $100k by end of year?");
        let outcomes = Vec::from_array(
            &env,
            [String::from_str(&env, "YES"), String::from_str(&env, "NO")],
        );
        let resolution_date = env.ledger().timestamp() + 3600;
        let oracle = Address::generate(&env);

        let proposal_id =
            client.propose_market(&proposer, &question, &outcomes, &resolution_date, &oracle);

        assert_eq!(proposal_id, 1);

        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.question, question);
        assert_eq!(proposal.approval_count, 0);
        assert!(!proposal.decided);

        // Approve 1
        client.approve_market(&m1, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.approval_count, 1);
        assert!(!proposal.decided);

        // Double approval from m1 should fail
        let res = client.try_approve_market(&m1, &proposal_id);
        assert!(res.is_err());

        // Approve 2 (reaches threshold of 2)
        client.approve_market(&m2, &proposal_id);
        let proposal = client.get_proposal(&proposal_id);
        assert_eq!(proposal.approval_count, 2);
        assert!(proposal.decided);

        // Market should be created
        let market = client.get_market(&1);
        assert_eq!(market.market_id, 1);
        assert_eq!(market.question, question);
        assert_eq!(market.status, MarketStatus::Active);
    }

    #[test]
    fn test_reject_proposal() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MarketFactory);
        let client = MarketFactoryClient::new(&env, &contract_id);

        let m1 = Address::generate(&env);
        let m2 = Address::generate(&env);
        let maintainers = Vec::from_array(&env, [m1.clone(), m2.clone()]);

        client.initialize(&maintainers, &2);

        let proposer = Address::generate(&env);
        let question = String::from_str(&env, "Spam Question");
        let outcomes = Vec::from_array(
            &env,
            [String::from_str(&env, "YES"), String::from_str(&env, "NO")],
        );
        let resolution_date = env.ledger().timestamp() + 3600;
        let oracle = Address::generate(&env);

        let proposal_id =
            client.propose_market(&proposer, &question, &outcomes, &resolution_date, &oracle);

        // Reject
        let reason = String::from_str(&env, "Spam proposal");
        client.reject_market(&m1, &proposal_id, &reason);

        let proposal = client.get_proposal(&proposal_id);
        assert!(proposal.decided);

        // Attempting to approve now should fail
        let res = client.try_approve_market(&m2, &proposal_id);
        assert!(res.is_err());
    }

    #[test]
    fn test_invalid_proposals() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MarketFactory);
        let client = MarketFactoryClient::new(&env, &contract_id);

        let m1 = Address::generate(&env);
        let maintainers = Vec::from_array(&env, [m1.clone()]);
        client.initialize(&maintainers, &1);

        let proposer = Address::generate(&env);
        let oracle = Address::generate(&env);

        // Empty question
        let question = String::from_str(&env, "");
        let outcomes = Vec::from_array(
            &env,
            [String::from_str(&env, "YES"), String::from_str(&env, "NO")],
        );
        let resolution_date = env.ledger().timestamp() + 3600;
        let res =
            client.try_propose_market(&proposer, &question, &outcomes, &resolution_date, &oracle);
        assert!(res.is_err());

        // Less than 2 outcomes
        let question = String::from_str(&env, "Valid question?");
        let outcomes = Vec::from_array(&env, [String::from_str(&env, "YES")]);
        let res =
            client.try_propose_market(&proposer, &question, &outcomes, &resolution_date, &oracle);
        assert!(res.is_err());

        // Past resolution date
        let outcomes = Vec::from_array(
            &env,
            [String::from_str(&env, "YES"), String::from_str(&env, "NO")],
        );
        let resolution_date = 0;
        let res =
            client.try_propose_market(&proposer, &question, &outcomes, &resolution_date, &oracle);
        assert!(res.is_err());
    }

    #[test]
    fn test_list_markets_pagination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, MarketFactory);
        let client = MarketFactoryClient::new(&env, &contract_id);

        let m1 = Address::generate(&env);
        let maintainers = Vec::from_array(&env, [m1.clone()]);
        client.initialize(&maintainers, &1);

        let proposer = Address::generate(&env);
        let oracle = Address::generate(&env);
        let outcomes = Vec::from_array(
            &env,
            [String::from_str(&env, "YES"), String::from_str(&env, "NO")],
        );
        let resolution_date = env.ledger().timestamp() + 3600;

        // Create 12 markets
        for _ in 1..=12 {
            let question = String::from_str(&env, "Question");
            let proposal_id =
                client.propose_market(&proposer, &question, &outcomes, &resolution_date, &oracle);
            client.approve_market(&m1, &proposal_id);
        }

        // List markets page 0
        let list_p0 = client.list_markets(&MarketStatus::Active, &0);
        assert_eq!(list_p0.len(), 10);

        // List markets page 1
        let list_p1 = client.list_markets(&MarketStatus::Active, &1);
        assert_eq!(list_p1.len(), 2);
    }
}
