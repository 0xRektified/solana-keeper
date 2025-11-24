
use crate::core::executor::Executor;
use crate::solana::state::TaskAccount;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use anyhow::Result;

pub struct ResolveExecutor {
    pub rpc_client: RpcClient,
    pub keypair: Keypair,
    pub program_id: Pubkey,
}

impl Executor<TaskAccount> for ResolveExecutor {
    fn execute(&self, state: &TaskAccount) -> Result<()> {
        todo!();
    }
}




