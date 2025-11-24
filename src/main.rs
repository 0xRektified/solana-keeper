#![warn(dead_code)]
#![warn(unused_imports)]

mod core;
mod solana;

use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::{env, thread::sleep, time::{self}};
use dotenv::dotenv;
use anyhow::Result;
use tokio;


use crate::{core::watcher::Watcher, solana::{executor::ResolveExecutor, trigger::TimestampTrigger}};

struct Config {
    rpc_url: String,
    polling_interval: u64,
    target_account: String,
}

impl Config {
    fn from_env() -> Result<(Config)> {
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
    let client = RpcClient::new(config.rpc_url);
    let interval_duration = time::Duration::from_secs(config.polling_interval);
    
    let trigger = TimestampTrigger{};

    let program_id = Pubkey::from_str_const(&config.target_account);
    let kp = Keypair::new();
    let executor = ResolveExecutor{
        rpc_client: client,
        keypair: kp,
        program_id: program_id,
    };
    
    let watcher = Watcher{
        trigger: Box::new(trigger),
        executor: Box::new(executor),
        duration: interval_duration
    };
    Ok(())
}
