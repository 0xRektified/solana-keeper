use crate::core::trigger::Trigger;
use crate::solana::state::TaskAccount;

pub struct TimestampTrigger {}

impl Trigger<TaskAccount> for TimestampTrigger {
    fn should_trigger(&self, task: &TaskAccount) -> bool {
        // Use on-chain block timestamp instead of system time
        // This ensures we're using the same clock the program uses
        if task.block_timestamp > task.end_at {
            return true;
        }
        false
    }
}
