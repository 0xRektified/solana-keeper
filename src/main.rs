use solana_client::rpc_client::RpcClient;
use std::{env, thread::sleep, time::{self}};
use dotenv::dotenv;
use anyhow::Result;

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

fn main() -> Result<()> {

    let config = Config::from_env()?;
    let client = RpcClient::new(config.rpc_url);
    let interval_duration = time::Duration::from_secs(config.polling_interval);
    loop {
        sleep(interval_duration);
        match client.get_latest_blockhash() {
            Ok(blockhash) => println!("Connected: {}", blockhash),
            Err(e) => println!("failed: {}", e),
        }
    }
}
