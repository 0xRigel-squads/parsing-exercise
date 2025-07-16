use std::collections::HashMap;

use crate::models::token_accounts::TokenAccountChange;
use chrono::NaiveDateTime;
use solana_pubkey::Pubkey;
use solana_signature::Signature;

use crate::transaction::transaction::UnifiedTransaction;

use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueueEntry {
    pub signature: Signature,
    pub network: i32,
    pub transaction: UnifiedTransaction,
    pub slot: i32,
    #[serde(deserialize_with = "deserialize_block_time")]
    pub block_time: i64,
    pub token_account_changes: HashMap<Pubkey, TokenAccountChange>,
}

fn deserialize_block_time<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // First try to deserialize as a normal Value
    let value_opt = serde_json::Value::deserialize(deserializer).ok();

    if let Some(value) = value_opt {
        match value {
            serde_json::Value::Number(num) => Ok(num.as_i64().unwrap_or(0)),
            serde_json::Value::String(s) => {
                match NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f") {
                    Ok(dt) => Ok(dt.and_utc().timestamp()),
                    Err(_) => Err(D::Error::custom(format!("Invalid datetime format: {}", s))),
                }
            }
            _ => Err(D::Error::custom(format!(
                "Expected number or string for block_time, got: {:?}",
                value
            ))),
        }
    } else {
        // If we failed to deserialize, provide a default value
        Ok(0)
    }
}

pub mod vectorize {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter::FromIterator;

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        let container: Vec<_> = target.into_iter().collect();
        serde::Serialize::serialize(&container, ser)
    }

    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(T::from_iter(container))
    }
}

impl Default for QueueEntry {
    /// default is mostly used for testing.
    fn default() -> Self {
        Self {
            signature: Signature::new_unique(),
            network: 0,
            transaction: UnifiedTransaction::default(),
            slot: 0,
            block_time: 0,
            token_account_changes: HashMap::new(),
        }
    }
}

impl QueueEntry {
    pub fn new(network: i32, signature: Signature, transaction: UnifiedTransaction) -> Self {
        let slot = transaction.slot as i32;
        Self {
            signature,
            network,
            transaction,
            slot,
            block_time: 0,
            token_account_changes: HashMap::new(),
        }
    }

    pub fn contains_changes(&self) -> bool {
        !self.token_account_changes.is_empty()
    }
}
