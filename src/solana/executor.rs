
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

const SEED_POOL: &[u8] = b"pool";


fn calculate_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut hasher = Sha256::new();
    hasher.update(preimage.as_bytes());
    let result = hasher.finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&result[..8]);
    discriminator
}

fn build_resolve_transaction(ctx: &ResolveExecutor, state: &TaskAccount )-> Result<u64> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 1000;

        let discriminator = calculate_discriminator("global", "resolve");
        let mut instruction_data = Vec::new();
        // Add Discriminator
        instruction_data.extend_from_slice(&discriminator);
        // Add argument
        instruction_data.push(0u8);
        // println!("Discriminator: {:?}", discriminator);

        let mut accounts = vec![
            AccountMeta::new(ctx.keypair.pubkey(), true),           // 0: signer
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
        // println!("program_identity_pda {}", program_identity_pda);
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
        // println!("pool count {}", &state.pool_count);

        for i in 0..state.pool_count {
            let (pool_pda, _) = Pubkey::find_program_address(
                &[
                    SEED_POOL,
                    &[i],
                    state.epoch.to_le_bytes(). as_ref()
                ],
                &state.program_id
            );
            // println!("pool_pda {}", &pool_pda);

            remaining_account.push(AccountMeta::new(pool_pda, false));
        }
        accounts.append(&mut remaining_account);
        let instruction = Instruction {
            program_id: state.program_id,
            accounts,
            data: instruction_data,
        };

        // Retry logic for epoch timing issues
        let mut last_error = None;
        for attempt in 1..=MAX_RETRIES {
            let recent_blockhash = ctx.rpc_client.get_latest_blockhash()?;

            let transaction = Transaction::new_signed_with_payer(
                &[instruction.clone()],
                Some(&ctx.keypair.pubkey()),
                &[&ctx.keypair],
                recent_blockhash,
            );

            match ctx.rpc_client.send_and_confirm_transaction(&transaction) {
                Ok(signature) => {
                    println!("Transaction successful: {}", signature);
                    return Ok(state.epoch + 1);
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES {
                        println!("Attempt {} failed, retrying in {}ms...", attempt, RETRY_DELAY_MS);
                        std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                    }
                }
            }
        }

        // All retries failed, return the last error
        Err(last_error.unwrap().into())
}

fn build_init_position_transaction(
    ctx: &ResolveExecutor,
    state: &TaskAccount,
    next_epoch: u64
) ->Result<()> {
    let discriminator = calculate_discriminator("global", "initialize_pool");
    let mut instruction_data = Vec::new();
    // Add Discriminator
    instruction_data.extend_from_slice(&discriminator);
    // Add argument create 4 pools
    instruction_data.push(4u8);


    let (epoch_result_pda, _bump) = Pubkey::find_program_address(
    &[
            b"epoch_result",
            next_epoch.to_le_bytes().as_ref()
        ],
        &state.program_id
    );

    println!("ctx.keypair.pubkey() {}", ctx.keypair.pubkey());
    println!("epoch_result_pda {}", epoch_result_pda);
    println!("state.config_pda {}", state.config_pda);

    let mut accounts = vec![
        AccountMeta::new(ctx.keypair.pubkey(), true),           // 0: signer
        AccountMeta::new(state.config_pda, false),              // 1: config
        AccountMeta::new(epoch_result_pda, false),        // 2: epoch_result
        AccountMeta::new(system_program::ID, false),            // 3: system_program
    ];

    let mut remaining_account = vec![];
    println!("pool count {}", &state.pool_count);

    for i in 0..state.pool_count {
        let (pool_pda, _) = Pubkey::find_program_address(
            &[
                SEED_POOL,
                &[i],
                next_epoch.to_le_bytes(). as_ref()
            ],
            &state.program_id
        );
        println!("pool_pda {}", &pool_pda);

        remaining_account.push(AccountMeta::new(pool_pda, false));
    }
    accounts.append(&mut remaining_account);

    let instruction = Instruction {
        program_id: state.program_id,
        accounts,
        data: instruction_data,
    };
    
    let recent_blockhash = ctx.rpc_client.get_latest_blockhash()?;
    
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ctx.keypair.pubkey()),
        &[&ctx.keypair],
        recent_blockhash,
    );
    
    let signature = ctx.rpc_client.send_and_confirm_transaction(&transaction)?;
    println!("Transaction successful: {}", signature);

    Ok(())
}

pub struct ResolveExecutor {
    pub rpc_client: Arc<RpcClient>,
    pub keypair: Keypair,
    pub program_id: Pubkey,
}

impl Executor<TaskAccount> for ResolveExecutor {
    fn execute(&self, state: &TaskAccount) -> Result<()> {
        // @todo improve Do not resolve if state is pending
        let next_epoch = build_resolve_transaction(self, state)?;
        build_init_position_transaction(self, state, next_epoch)?;
        Ok(())
    }
}




