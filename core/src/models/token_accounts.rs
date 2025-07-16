use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;
use solana_signature::Signature;


// Token account type thats compatible with Diesel / Postgres
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TokenAccountChange {
    pub address: String,
    pub network: i32,
    pub owner: String,
    pub mint: String,
    pub amount: String,
    pub decimals: i32,
    pub ui_amount: String,
    pub delegate: Option<String>,
    pub is_frozen: bool,
    pub last_updated_signature: String,
    pub last_updated_slot: i64,
}
