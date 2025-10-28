use crate::queue_entry::QueueEntry;
use async_trait::async_trait;
use fxhash::FxHashSet;
use solana_pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

/// Local Redis emulator that simulates the performance characteristics of real Redis
/// without requiring actual Redis infrastructure
#[derive(Debug, Clone)]
pub struct LocalRedisEmulator {
    // Simulated storage
    relevant_account_cache: RelevantAccountCache,
}

#[derive(Debug, Clone)]
pub struct RelevantAccountCache {
    token_account_owners: Option<FxHashSet<Pubkey>>,
}

impl Default for RelevantAccountCache {
    fn default() -> Self {
        Self::new()
    }
}

impl RelevantAccountCache {
    fn new() -> Self {
        Self {
            token_account_owners: None,
        }
    }
}

impl LocalRedisEmulator {
    pub fn new() -> Self {
        Self {
            relevant_account_cache: RelevantAccountCache::new(),
        }
    }

    /// Batch check token account owners - emulates piping the queries into
    /// redis via a single call
    pub async fn batch_check_token_account_owners(
        &self,
        owners: &[Pubkey],
    ) -> Result<Vec<bool>, anyhow::Error> {
        // Simulate cache lookup time - 0.5ms for batch operations
        sleep(Duration::from_micros(500)).await;

        match &self.relevant_account_cache.token_account_owners {
            Some(token_owners) => Ok(owners
                .iter()
                .map(|owner| token_owners.contains(owner))
                .collect()),
            None => {
                // If cache not loaded, return all false (no matches)
                Ok(vec![false; owners.len()])
            }
        }
    }

    /// Populate the emulator with realistic data sizes
    pub fn populate_with_realistic_data(&mut self) {
        // Generate 125k unique token account owners
        let mut token_owners = FxHashSet::default();
        for _ in 0..125_000 {
            token_owners.insert(Pubkey::new_unique());
        }

        // Update the cache with realistic data
        self.relevant_account_cache.token_account_owners = Some(token_owners);

        info!("Populated Redis emulator with 125k token owners");
    }
}

// Type alias for backward compatibility
pub type SmartAccountRedisClient = LocalRedisEmulator;
