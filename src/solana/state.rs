use borsh::BorshDeserialize;

#[derive(BorshDeserialize, Debug)]
pub struct TaskAccount {
    pub start_at: u64,
    pub end_at: u64,
    pub state: u8,
}