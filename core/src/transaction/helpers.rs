use std::collections::HashMap;

use solana_pubkey::Pubkey;

use super::transaction::{CompiledInstruction, InnerInstruction, UnifiedTransaction};

impl UnifiedTransaction {
    /// Get the complete account keys from the transaction, including those from address table lookups
    pub fn get_account_keys(&self) -> Vec<Pubkey> {
        // Start with the main account keys from the message
        let mut all_keys = match &self.transaction.message {
            Some(message) => message.account_keys.clone(),
            None => Vec::new(),
        };

        // Add the addresses from address table lookups
        all_keys.extend(self.meta.loaded_writable_addresses.iter().cloned());
        all_keys.extend(self.meta.loaded_readonly_addresses.iter().cloned());

        all_keys
    }

    /// Filter instructions by program IDs
    /// Returns all instructions (including inner instructions) that call any of the specified program IDs
    /// If program_ids is empty, returns all instructions
    pub fn get_all_instructions_by_programs(
        &self,
        program_ids: Vec<Pubkey>,
    ) -> Vec<(usize, CompiledInstruction)> {
        // Get all instructions in the transaction
        let all_instructions = self.get_all_instructions_ordered();

        // Get the account keys
        let account_keys = self.get_account_keys();

        // Filter instructions by program IDs
        all_instructions
            .into_iter()
            .enumerate()
            .filter(|(_idx, ix)| program_ids.contains(&account_keys[ix.program_id_index as usize]))
            .collect()
    }

    /// Gets all instructions from a transaction grouped by program ID
    pub fn get_instructions_by_program_id(&self) -> HashMap<Pubkey, Vec<CompiledInstruction>> {
        let mut instructions_by_program = HashMap::new();

        // Get message to access the main instructions
        if let Some(message) = &self.transaction.message {
            // Get main and inner instructions
            let main_instructions = &message.instructions;
            let inner_instructions = &self.meta.inner_instructions;
            let account_keys = self.get_account_keys();

            // Process each main instruction and any associated inner instructions in sequence
            for (ix_index, main_ix) in main_instructions.iter().enumerate() {
                // Get program ID for main instruction
                if let Some(program_pubkey) = account_keys.get(main_ix.program_id_index as usize) {
                    instructions_by_program
                        .entry(*program_pubkey)
                        .or_insert_with(Vec::new)
                        .push(main_ix.clone());
                }

                // Find and process any inner instructions associated with this main instruction
                if let Some(inner_group) = inner_instructions
                    .iter()
                    .find(|inner| inner.index == ix_index as u32)
                {
                    // Add all inner instructions for this main instruction in sequence
                    for inner_ix in &inner_group.instructions {
                        // Get program ID for inner instruction
                        if let Some(program_pubkey) =
                            account_keys.get(inner_ix.program_id_index as usize)
                        {
                            let compiled_inner = CompiledInstruction {
                                program_id_index: inner_ix.program_id_index,
                                accounts: inner_ix.accounts.clone(),
                                data: inner_ix.data.clone(),
                            };

                            instructions_by_program
                                .entry(*program_pubkey)
                                .or_insert_with(Vec::new)
                                .push(compiled_inner);
                        }
                    }
                }
            }
        }

        instructions_by_program
    }

    pub fn get_all_inner_instructions_by_programs(
        &self,
        program_ids: &[Pubkey],
    ) -> Vec<(u32, InnerInstruction)> {
        let all_inner_instructions = self.get_all_inner_instructions();
        let account_keys = self.get_account_keys();

        let filtered_inner_instructions = all_inner_instructions
            .iter()
            .filter(|(_, ix)| program_ids.contains(&account_keys[ix.program_id_index as usize]))
            .cloned()
            .collect();

        filtered_inner_instructions
    }

    /// Gets all instructions from a transaction in order, including inner instructions, flattened into a single vector
    pub fn get_all_instructions_ordered(&self) -> Vec<CompiledInstruction> {
        let mut all_instructions = Vec::new();

        // Get message to access the main instructions
        if let Some(message) = &self.transaction.message {
            // Get main and inner instructions
            let main_instructions = &message.instructions;
            let inner_instructions = &self.meta.inner_instructions;

            // Process each main instruction and any associated inner instructions in sequence
            for (ix_index, main_ix) in main_instructions.iter().enumerate() {
                // Add the main instruction first
                all_instructions.push(main_ix.clone());

                // Find and process any inner instructions associated with this main instruction
                if let Some(inner_group) = inner_instructions
                    .iter()
                    .find(|inner| inner.index == ix_index as u32)
                {
                    // Add all inner instructions for this main instruction in sequence
                    for inner_ix in &inner_group.instructions {
                        // Convert inner instruction to CompiledInstruction and add to our list
                        let compiled_inner = CompiledInstruction {
                            program_id_index: inner_ix.program_id_index,
                            accounts: inner_ix.accounts.clone(),
                            data: inner_ix.data.clone(),
                        };
                        all_instructions.push(compiled_inner);
                    }
                }
            }
        }

        all_instructions
    }
    /// Gets all inner instructions from a transaction, flattened into a single vector
    pub fn get_all_inner_instructions(&self) -> Vec<(u32, InnerInstruction)> {
        self.meta
            .inner_instructions
            .iter()
            .flat_map(|inner| {
                inner
                    .instructions
                    .iter()
                    .map(move |instruction| (inner.index, instruction.clone()))
            })
            .collect()
    }
}
