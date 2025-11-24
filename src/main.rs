use solana_client::rpc_client::RpcClient;
use std::env;
use dotenv::dotenv;
use anyhow::Result;

fn main() -> Result<()> {
    dotenv().ok();

    let rpc_url = env::var("RPC_URL")
        .expect("RPC_URL must be set in .env file");
    let client = RpcClient::new(rpc_url);

    match client.get_latest_blockhash() {
        Ok(blockhash) => println!("Connected: {}", blockhash),
        Err(e) => println!("failed: {}", e),
    }
    println!("Hello, world!");
    Ok(())
}
