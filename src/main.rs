use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{info, warn};

use indexer_core::parsing::simple_parser::parse_transaction_simple;
use indexer_core::redis::SmartAccountRedisClient;
use indexer_core::transaction::transaction::UnifiedTransaction;

#[derive(Parser, Debug)]
#[command(name = "parsing-exercise")]
#[command(about = "Benchmark token program parser with real transactions")]
struct Args {
    /// Input file path with captured transactions
    #[arg(long, default_value = "mainnet_transactions.json")]
    input_file: String,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapturedTransaction {
    transaction: UnifiedTransaction,
    captured_at: DateTime<Utc>,
    slot: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionCapture {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_seconds: u64,
    transaction_count: usize,
    transactions: Vec<CapturedTransaction>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Loading transactions from: {}", args.input_file);

    // Load captured transactions
    let file_content = std::fs::read_to_string(&args.input_file)?;
    let capture: TransactionCapture = serde_json::from_str(&file_content)?;

    let transactions_to_process = capture.transactions.len();

    info!("Loaded {} transactions", capture.transactions.len());

    // Initialize Redis emulator with realistic data
    let mut redis_client = SmartAccountRedisClient::new();
    redis_client.populate_with_realistic_data();

    info!("Starting token program parser benchmark...");

    let start_time = Instant::now();
    let mut successful_parses = 0;
    let mut failed_parses = 0;

    // Process transactions
    for (i, captured_tx) in capture
        .transactions
        .iter()
        .take(transactions_to_process)
        .enumerate()
    {
        match parse_transaction_simple(
            &redis_client,
            1, // mainnet
            captured_tx.transaction.clone(),
        )
        .await
        {
            Ok(Some(_queue_entry)) => {
                successful_parses += 1;
            }
            Ok(None) => {
                // Transaction not relevant, this is normal
            }
            Err(e) => {
                failed_parses += 1;
                warn!("Failed to parse transaction {}: {:?}", i, e);
            }
        }

        // Log progress every 1000 transactions
        if (i + 1) % 1000 == 0 {
            info!("Processed {} transactions", i + 1);
        }
    }

    //#### After parsing, the queue entry is populated with the token account changes and potentially enqueued and sent off to the consumer
    let duration = start_time.elapsed();
    let tps = transactions_to_process as f64 / duration.as_secs_f64();

    info!("=== BENCHMARK RESULTS ===");
    info!("Total transactions processed: {}", transactions_to_process);
    info!("Successful parses: {}", successful_parses);
    info!("Failed parses: {}", failed_parses);
    info!("Total duration: {:.2}s", duration.as_secs_f64());
    info!("Transactions per second (TPS): {:.2}", tps);
    info!(
        "Average time per transaction: {:.2}ms",
        duration.as_millis() as f64 / transactions_to_process as f64
    );

    Ok(())
}
