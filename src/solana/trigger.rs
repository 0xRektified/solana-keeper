use crate::core::trigger::Trigger;
use crate::solana::state::TaskAccount;

pub struct TimestampTrigger {}

impl Trigger<TaskAccount> for TimestampTrigger {
    fn should_trigger(&self, task: TaskAccount) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if current_time > task.start_at {
            return true;
        }
        false
    }
}
