mod core;
mod solana;

use borsh::BorshDeserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey};
use std::{env, time::{self}};
use std::str::FromStr;
use dotenv::dotenv;
use anyhow::Result;
use std::sync::Arc;
use std::cell::RefCell;
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
    program_id: String,
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
        
            program_id: env::var("PROGRAM_ID")
                .expect("PROGRAM_ID must be set in .env file"),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {

    let config = Config::from_env()?;
    let client = Arc::new(RpcClient::new(config.rpc_url));
    let interval_duration = time::Duration::from_secs(config.polling_interval);
    
    let trigger = TimestampTrigger{
        rpc_client: Arc::clone(&client),
    };

    let program_id = Pubkey::from_str(&config.program_id)?;
    let (config_pda, _bump) = Pubkey::find_program_address(
        &[b"config"],
        &program_id
    );
    
    let keypair_path = std::env::var("KEYPAIR_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not set");
            format!("{}/.config/solana/keeper-keypair.json", home)
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

    let config_pda_account = client.get_account(&config_pda)?;
    println!("{}", &config_pda_account.data.len());
    let config_state = ConfigAccount::try_from_slice(&config_pda_account.data[8..])?;
    println!("config_state: {:?}", config_state);
    let epoch = config_state.current_epoch;


    let (epoch_result_pda, _bump) = Pubkey::find_program_address(
    &[
            b"epoch_result",
            epoch.to_le_bytes().as_ref()
        ],
        &program_id
    );
    let epoch_result_pda_account = client.get_account(&epoch_result_pda)?;
    let epoch_state = EpochAccount::try_from_slice(&epoch_result_pda_account.data[8..])?;
    println!("epoch_state: {:?}", epoch_state);

    let end_at= epoch_state.end_at;


    let custom_pda = env::var("CUSTOM_PDA")
    .ok()
    .and_then(|s| Pubkey::from_str(&s).ok());

    let state = RefCell::new(TaskAccount {
        config_pda,
        epoch_result_pda,
        program_id,
        epoch,
        end_at,
        epoch_result_state: epoch_state.epoch_result_state,
        pool_count: epoch_state.pool_count,
        custom_pda,
    });

    watcher.run(&state).await?;
    Ok(())
}
