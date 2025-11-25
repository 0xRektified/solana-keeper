// #![warn(dead_code)]
// #![warn(unused_imports)]

mod core;
mod solana;

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::{env, time::{self}};
use std::str::FromStr;
use dotenv::dotenv;
use anyhow::Result;
use std::sync::Arc;
use crate::{
    core::watcher::Watcher,
    solana::{
        executor::ResolveExecutor,
        state::{
            ConfigAccount,
            EpochAccount,
            TaskAccount,
        },
        trigger::TimestampTrigger
    }
};
use solana_sdk::signature::read_keypair_file;

struct Config {
    rpc_url: String,
    polling_interval: u64,
    target_account: String,
}

impl Config {
    fn from_env() -> Result<Config> {
        dotenv().ok();

        Ok(Config {
            rpc_url: env::var("RPC_URL")
            .expect("RPC_URL must be set in .env file"),

            polling_interval: env::var("POLL_INTERVAL_SECS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .expect("POLL_INTERVAL_SECS should be a number"),
        
            target_account: env::var("TARGET_ACCOUNT")
                .expect("TARGET_ACCOUNT must be set in .env file"),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {

    let config = Config::from_env()?;
    let client = Arc::new(RpcClient::new(config.rpc_url));
    let interval_duration = time::Duration::from_secs(config.polling_interval);
    
    let trigger = TimestampTrigger{};

    let program_id = Pubkey::from_str(&config.target_account)?;
    let (config_pda, _bump) = Pubkey::find_program_address(
        &[b"config"],
        &program_id
    );
    let keypair_path = std::env::var("KEYPAIR_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not set");
            format!("{}/.config/solana/id.json", home)
        });

    let kp = read_keypair_file(&keypair_path)
        .expect("Failed to read keypair file");

    let executor = ResolveExecutor{
        rpc_client: Arc::clone(&client),
        keypair: kp,
        program_id: program_id,
    };
    
    let watcher = Watcher{
        trigger: Box::new(trigger),
        executor: Box::new(executor),
        duration: interval_duration
    };

    let fetch_state = || -> Result<TaskAccount> {
        let custom_pda = env::var("CUSTOM_PDA")
            .ok()
            .and_then(|s| Pubkey::from_str(&s).ok());
        let account = client.get_account(&config_pda)?;
        // println!("len: {}", account.data.len());
        let config_state = ConfigAccount::try_from_slice(&account.data[8..])?;
        // println!("config_state: {:?}", config_state);

        let (epoch_result_pda, _bump) = Pubkey::find_program_address(
        &[
            b"epoch_result", config_state.current_epoch.to_le_bytes().as_ref()
            ],
            &program_id
        );
        let account = client.get_account(&epoch_result_pda)?;
        // println!("Len: {}", account.data.len());
        let epoch_state = EpochAccount::try_from_slice(&account.data[8..])?;
        // println!("epoch_state: {:?}", epoch_state);
        let task_account = TaskAccount{
            config_pda: config_pda,
            epoch_result_pda: epoch_result_pda,
            program_id: program_id,
            epoch: config_state.current_epoch,
            end_at: epoch_state.end_at,
            epoch_result_state: epoch_state.epoch_result_state,
            pool_count: epoch_state.pool_count,
            custom_pda: custom_pda,
        };
        Ok(task_account)
    };
    watcher.run(fetch_state).await?;
    Ok(())
}
