
use crate::core::executor::Executor;
use crate::solana::state::{TaskAccount, EpochAccount, EpochResultState};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcSendTransactionConfig, CommitmentConfig};
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
use borsh::BorshDeserialize;

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

fn build_resolve_transaction(ctx: &ResolveExecutor, state: &mut TaskAccount) -> Result<()> {
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

        let recent_blockhash = ctx.rpc_client.get_latest_blockhash()?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction.clone()],
            Some(&ctx.keypair.pubkey()),
            &[&ctx.keypair],
            recent_blockhash,
        );
        let config = RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        };
        let signature = ctx.rpc_client.send_and_confirm_transaction_with_spinner_and_config(
            &transaction,
            CommitmentConfig::confirmed(),
            config,
        )?;
        println!("Resolve Transaction successful: {}", signature);

        Ok(())
}

fn build_init_position_transaction(
    ctx: &ResolveExecutor,
    state: &mut TaskAccount,
) ->Result<()> {
    let discriminator = calculate_discriminator("global", "initialize_pool");
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&discriminator);
    instruction_data.push(4u8);

    let new_epoch = state.epoch + 1;
    println!("build_init_position_transaction new_epoch {}", new_epoch);
    let (epoch_result_pda, _bump) = Pubkey::find_program_address(
    &[
            b"epoch_result",
            new_epoch.to_le_bytes().as_ref()
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
                new_epoch.to_le_bytes().as_ref()
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

    let config = RpcSendTransactionConfig {
        skip_preflight: true,
        ..Default::default()
    };
    let signature = ctx.rpc_client.send_and_confirm_transaction_with_spinner_and_config(
        &transaction,
        CommitmentConfig::confirmed(),
        config,
    )?;
    println!("Init pool Transaction successful: {}", signature);

    state.epoch = new_epoch;

    Ok(())
}

pub struct ResolveExecutor {
    pub rpc_client: Arc<RpcClient>,
    pub keypair: Keypair,
    #[allow(dead_code)]
    pub program_id: Pubkey,
}

impl Executor<TaskAccount> for ResolveExecutor {
    fn execute(&self, state: &mut TaskAccount) -> Result<()> {
        println!("Executing: epoch {} state {:?}", state.epoch, state.epoch_result_state);

        match state.epoch_result_state {
            EpochResultState::Active => {
                build_resolve_transaction(self, state)?;
                self.refresh_epoch_state(state)?;
            }
            EpochResultState::Resolved => {
                build_init_position_transaction(self, state)?;
                self.refresh_epoch_state(state)?;
            }
            EpochResultState::Pending => {
                // VRF callback pending - refresh state to check if it completed
                println!("Epoch {} is Pending (waiting for VRF callback), refreshing state...", state.epoch);
                self.refresh_epoch_state(state)?;
                // If VRF completed and state is now Resolved, initialize pools
                if state.epoch_result_state == EpochResultState::Resolved {
                    println!("VRF callback completed, initializing pools...");
                    build_init_position_transaction(self, state)?;
                    self.refresh_epoch_state(state)?;
                }
            }
        }

        Ok(())
    }
}

impl ResolveExecutor {
    fn refresh_epoch_state(&self, state: &mut TaskAccount) -> Result<()> {
        let (epoch_result_pda, _bump) = Pubkey::find_program_address(
            &[
                b"epoch_result",
                state.epoch.to_le_bytes().as_ref()
            ],
            &state.program_id
        );

        // Use confirmed commitment to get fresh data after transaction
        let epoch_result_account = self.rpc_client.get_account_with_commitment(
            &epoch_result_pda,
            CommitmentConfig::confirmed()
        )?.value.ok_or_else(|| anyhow::anyhow!("Account not found: {}", epoch_result_pda))?;
        let epoch_state = EpochAccount::try_from_slice(&epoch_result_account.data[8..])?;

        state.epoch_result_pda = epoch_result_pda;
        state.end_at = epoch_state.end_at;
        state.epoch_result_state = epoch_state.epoch_result_state;
        state.pool_count = epoch_state.pool_count;

        println!("Refreshed state for epoch {}: end_at={}, state={:?}",
            state.epoch, state.end_at, state.epoch_result_state);

        Ok(())
    }
}




