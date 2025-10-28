use crate::models::token_accounts::TokenAccountChange;
use crate::{
    parsing::parser_trait::{ExtendQueueEntry, ParsingResult, ProgramParser},
    redis::SmartAccountRedisClient,
    transaction::transaction::{CompiledInstruction, TokenAmount},
    QueueEntry,
};
use async_trait::async_trait;
use solana_pubkey::Pubkey;
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::collections::HashMap;

pub struct TokenProgramParser {}

impl Default for TokenProgramParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenProgramParser {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct TokenProgramParsingResult {
    pub token_account_changes: HashMap<Pubkey, TokenAccountChange>,
}

impl Default for TokenProgramParsingResult {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenProgramParsingResult {
    pub fn new() -> Self {
        Self {
            token_account_changes: HashMap::new(),
        }
    }
}

impl ExtendQueueEntry for TokenProgramParsingResult {
    fn extend_queue_entry(self, queue_entry: &mut QueueEntry) {
        queue_entry
            .token_account_changes
            .extend(self.token_account_changes);
    }
}

// Helper function to create TokenAccountChange
fn create_token_account_change(
    address: Pubkey,
    network: i32,
    owner: Pubkey,
    mint: Pubkey,
    amount: u64,
    decimals: i32,
    ui_amount: String,
    delegate: Option<Pubkey>,
    is_frozen: bool,
    last_updated_signature: solana_signature::Signature,
    last_updated_slot: i64,
) -> TokenAccountChange {
    TokenAccountChange {
        address: address.to_string(),
        network,
        owner: owner.to_string(),
        mint: mint.to_string(),
        amount: amount.to_string(),
        decimals,
        ui_amount: ui_amount.to_string(),
        delegate: delegate.map(|d| d.to_string()),
        is_frozen,
        last_updated_signature: last_updated_signature.to_string(),
        last_updated_slot,
    }
}

#[async_trait]
impl ProgramParser for TokenProgramParser {
    async fn parse_transaction(
        &self,
        queue_entry: &QueueEntry,
        network: i32,
        redis_client: &SmartAccountRedisClient,
        instructions: &HashMap<Pubkey, Vec<CompiledInstruction>>,
    ) -> Result<Option<ParsingResult>, anyhow::Error> {
        let account_keys = queue_entry.transaction.get_account_keys();
        let post_token_balances = queue_entry.transaction.meta.post_token_balances.clone();

        // Get all token program instructions
        let all_token_program_instructions = match instructions.get(&TOKEN_PROGRAM_ID) {
            Some(instructions) => instructions,
            None => return Ok(None),
        };

        let mut result = TokenProgramParsingResult::new();

        // Collect all owners to check for batch filtering. Saves us multiple redis calls per txn.
        let mut all_owners_to_check = Vec::new();

        for ix in all_token_program_instructions {
            // Skip if we don't have enough accounts
            if ix.accounts.len() < 2 {
                continue;
            }

            // Assume account 0 is the token account
            let token_account_index = ix.accounts[0] as usize;
            if token_account_index >= account_keys.len() {
                continue;
            }

            let token_account_address = account_keys[token_account_index];

            // Assume account 1 is the owner/mint (simplified assumption)
            let owner_index = ix.accounts[1] as usize;
            if owner_index >= account_keys.len() {
                continue;
            }

            let owner_address = account_keys[owner_index];

            // Try to find post balance for this account
            let maybe_post_balance = post_token_balances
                .iter()
                .find(|balance| balance.account_index == token_account_index as u32);

            let (amount, decimals, ui_amount, owner, mint) =
                if let Some(balance) = maybe_post_balance {
                    let balances = match &balance.ui_token_amount {
                        Some(token_amount) => token_amount,
                        None => &TokenAmount::default(),
                    };
                    (
                        balances.amount,
                        balances.decimals as i32,
                        balances.ui_amount_string.clone(),
                        balance.owner,
                        balance.mint,
                    )
                } else {
                    // Default values if no balance found
                    (
                        0,
                        0,
                        "0".to_string(),
                        owner_address,
                        owner_address,
                    )
                };

            // Add owner to check list for batch processing
            all_owners_to_check.push(owner);

            // Create change entry
            let entry = create_token_account_change(
                token_account_address,
                network,
                owner,
                mint,
                amount,
                decimals,
                ui_amount,
                None,  // No delegate for simplified version
                false, // Not frozen for simplified version
                queue_entry.signature,
                queue_entry.slot as i64,
            );

            result
                .token_account_changes
                .insert(token_account_address, entry);
        }

        // Batch filter token account changes based on owner relevance
        if !all_owners_to_check.is_empty() {
            // Use batch check function for better performance
            let owner_relevance = redis_client
                .batch_check_token_account_owners(&all_owners_to_check)
                .await
                .unwrap_or_default();

            let mut filtered_changes = HashMap::new();
            for (address, change) in result.token_account_changes {
                // Parse the owner string back to Pubkey for checking
                if let Ok(owner_pubkey) = change.owner.parse::<Pubkey>() {
                    // Find the index of this owner in the all_owners_to_check list
                    if let Some(owner_index) =
                        all_owners_to_check.iter().position(|&x| x == owner_pubkey)
                    {
                        if owner_relevance.get(owner_index).copied().unwrap_or(false) {
                            filtered_changes.insert(address, change);
                        }
                    }
                }
            }
            result.token_account_changes = filtered_changes;
        }

        if result.token_account_changes.is_empty() {
            return Ok(None);
        }

        Ok(Some(ParsingResult::Token(result)))
    }
}
