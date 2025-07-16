use crate::{
    parsing::{
        parser_trait::{ExtendQueueEntry, ProgramParser},
        token_program::parser::TokenProgramParser,
    },
    transaction::transaction::UnifiedTransaction,
    QueueEntry, SmartAccountRedisClient,
};

/// Simplified parser that only runs the token program parser
pub async fn parse_transaction_simple(
    redis_client: &SmartAccountRedisClient,
    network: i32,
    transaction: UnifiedTransaction,
) -> Result<Option<QueueEntry>, anyhow::Error> {
    // Gets all instructions by program id
    let instructions = transaction.get_instructions_by_program_id();

    let mut state_and_transaction_changes =
        QueueEntry::new(network, transaction.signature, transaction);

    let token_parser = TokenProgramParser::new();

    // Only run the token program parser
    let parse_result = token_parser
        .parse_transaction(
            &state_and_transaction_changes,
            network,
            redis_client,
            &instructions,
        )
        .await;

    // Extend the queue entry with the parsing result
    match parse_result {
        Ok(Some(parsing_result)) => {
            parsing_result.extend_queue_entry(&mut state_and_transaction_changes);
            tracing::debug!("Token parser completed successfully");
        }
        Ok(None) => {
            tracing::debug!("Token parser found no relevant changes");
        }
        Err(e) => {
            tracing::error!("Token parser error: {}", e);
            return Err(e);
        }
    }

    // Check if we have any relevant changes
    if state_and_transaction_changes
        .token_account_changes
        .is_empty()
    {
        Ok(None)
    } else {
        Ok(Some(state_and_transaction_changes))
    }
}
