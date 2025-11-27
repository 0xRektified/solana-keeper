use crate::core::trigger::Trigger;
use crate::solana::state::TaskAccount;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::CommitmentConfig;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

pub struct TimestampTrigger {
    pub rpc_client: Arc<RpcClient>,
}

impl Trigger<TaskAccount> for TimestampTrigger {
    fn should_trigger(&self, task: &TaskAccount) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;
        println!("now: {:?}", now);

        if now > task.end_at {
            let slot = self.rpc_client.get_slot_with_commitment(CommitmentConfig::confirmed())?;

            let block_timestamp = match self.rpc_client.get_block_time(slot) {
                Ok(timestamp) => timestamp as i64,
                Err(e) => {
                    println!("Warning: Could not get block time for slot {}: {}", slot, e);
                    return Ok(false);
                }
            };

            println!("block_timestamp: {}, end_at: {}", block_timestamp, task.end_at);

            if block_timestamp > task.end_at {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
