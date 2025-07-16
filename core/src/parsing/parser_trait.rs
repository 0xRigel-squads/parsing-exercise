use std::collections::HashMap;

use async_trait::async_trait;
use solana_pubkey::Pubkey;

use super::token_program::parser::TokenProgramParsingResult;

use crate::{
    redis::SmartAccountRedisClient, transaction::transaction::CompiledInstruction, QueueEntry,
};

pub enum ParsingResult {
    Token(TokenProgramParsingResult),
}

pub trait ExtendQueueEntry {
    fn extend_queue_entry(self, queue_entry: &mut QueueEntry);
}

impl ExtendQueueEntry for ParsingResult {
    fn extend_queue_entry(self, queue_entry: &mut QueueEntry) {
        match self {
            ParsingResult::Token(result) => result.extend_queue_entry(queue_entry),
        }
    }
}
#[async_trait]
pub trait ProgramParser: Send + Sync {
    /// Parse a transaction and update the QueueEntry with program-specific data
    async fn parse_transaction(
        &self,
        queue_entry: &QueueEntry,
        network: i32,
        redis_client: &SmartAccountRedisClient,
        instructions: &HashMap<Pubkey, Vec<CompiledInstruction>>,
    ) -> Result<Option<ParsingResult>, anyhow::Error>;
}
