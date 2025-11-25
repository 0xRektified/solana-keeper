
use crate::core::executor::Executor;
use crate::solana::state::TaskAccount;
use solana_client::rpc_client::RpcClient;
use anyhow::Result;
use std::sync::Arc;
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    transaction::Transaction,
    signer::Signer,
    pubkey::Pubkey,
    signature::Keypair,
};
use solana_sdk_ids::system_program;


fn calculate_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut hasher = Sha256::new();
    hasher.update(preimage.as_bytes());
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

pub struct ResolveExecutor {
    pub rpc_client: Arc<RpcClient>,
    pub keypair: Keypair,
    pub program_id: Pubkey,
}

impl Executor<TaskAccount> for ResolveExecutor {
    fn execute(&self, state: &TaskAccount) -> Result<()> {
        let discriminator = calculate_discriminator("global", "resolve");
        let mut instruction_data = Vec::new();
        // Add Discriminator
        instruction_data.extend_from_slice(&discriminator);
        // Add argument
        instruction_data.push(0u8);
        println!("Discriminator: {:?}", discriminator);
        const SEED_POOL: &[u8] = b"pool";

        let mut accounts = vec![
            AccountMeta::new(self.keypair.pubkey(), true),           // 0: signer
            AccountMeta::new(state.config_pda, false),               // 1: config
            AccountMeta::new(state.epoch_result_pda, false),         // 2: epoch_result
        ];

        // 3: oracle_queue (required, from CUSTOM_PDA env or hardcoded)
        if let Some(custom_pda) = state.custom_pda {
            accounts.push(AccountMeta::new(custom_pda, false));
        }

        accounts.push(AccountMeta::new(system_program::ID, false));  // 4: system_program

        // 5: program_identity PDA
        let (program_identity_pda, _) = Pubkey::find_program_address(
            &[b"identity"],
            &state.program_id
        );
        accounts.push(AccountMeta::new_readonly(program_identity_pda, false));

        // 6: vrf_program
        accounts.push(AccountMeta::new_readonly(
            Pubkey::from_str_const("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz"),
            false
        ));

        // 7: slot_hashes sysvar
        accounts.push(AccountMeta::new_readonly(
            Pubkey::from_str_const("SysvarS1otHashes111111111111111111111111111"),
            false
        ));

        AccountMeta::new(system_program::ID, false);

        let mut remaining_account = vec![];

        for i in 0..state.pool_count {
            let (pool_pda, _) = Pubkey::find_program_address(
                &[
                    SEED_POOL,
                    &[i],
                    state.epoch.to_le_bytes(). as_ref()
                ],
                &state.program_id
            );
            remaining_account.push(AccountMeta::new(pool_pda, false));
        }
        accounts.append(&mut remaining_account);
        let instruction = Instruction {
            program_id: state.program_id,
            accounts,
            data: instruction_data,
        };
        
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );
        
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        println!("Transaction successful: {}", signature);
        
        Ok(())
    }
}




