use serde::{Deserialize, Serialize};
use solana_pubkey::Pubkey;
use solana_signature::Signature;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedTransaction {
    pub signature: Signature,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub transaction: Transaction,
    pub meta: TransactionStatusMeta,
    pub index: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Transaction {
    pub signatures: Vec<Signature>,
    pub message: Option<Message>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub header: Option<MessageHeader>,
    pub account_keys: Vec<Pubkey>,
    pub recent_blockhash: Vec<u8>,
    pub instructions: Vec<CompiledInstruction>,
    pub versioned: bool,
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MessageHeader {
    pub num_required_signatures: u32,
    pub num_readonly_signed_accounts: u32,
    pub num_readonly_unsigned_accounts: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledInstruction {
    pub program_id_index: u32,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageAddressTableLookup {
    pub account_key: Pubkey,
    pub writable_indexes: Vec<u8>,
    pub readonly_indexes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TransactionStatusMeta {
    pub fee: u64,
    pub pre_balances: Vec<u64>,
    pub post_balances: Vec<u64>,
    pub inner_instructions: Vec<InnerInstructions>,
    pub pre_token_balances: Vec<TokenBalance>,
    pub post_token_balances: Vec<TokenBalance>,
    pub loaded_writable_addresses: Vec<Pubkey>,
    pub loaded_readonly_addresses: Vec<Pubkey>,
    /// Sum of compute units consumed by all instructions.
    /// Available since Solana v1.10.35 / v1.11.6.
    /// Set to `None` for txs executed on earlier versions.
    pub compute_units_consumed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerInstructions {
    pub index: u32,
    pub instructions: Vec<InnerInstruction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerInstruction {
    pub program_id_index: u32,
    pub accounts: Vec<u8>,
    pub data: Vec<u8>,
    /// Invocation stack height of an inner instruction.
    /// Available since Solana v1.14.6
    /// Set to `None` for txs executed on earlier versions.
    pub stack_height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TokenBalance {
    pub account_index: u32,
    pub mint: Pubkey,
    pub ui_token_amount: Option<TokenAmount>,
    pub owner: Pubkey,
    pub program_id: Pubkey,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TokenAmount {
    pub ui_amount: f64,
    pub decimals: u32,
    pub amount: u64,
    pub ui_amount_string: String,
}

impl UnifiedTransaction {}
