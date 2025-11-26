use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum WeightModel {
    Constant,      // weight = 1
    TimeBased,     // weight = age (current_time - created_at)
}

#[derive(BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ResolutionType {
    Admin,                              // Admin resolves manually
    Oracle, // Future: Oracle resolution
    // Timelock { resolve_at: i64 },    // Future: Time-based resolution
}

#[derive(BorshDeserialize, Debug)]
pub struct ConfigAccount {
    pub _admin: Pubkey,
    pub _resolver: Pubkey,
    pub current_epoch: u64,
    pub _total_positions_minted: u64,
    pub _position_price: u64,
    pub _remaining_total_position: u64,
    pub _allowed_mint: Pubkey,
    pub _treasury_ata: Pubkey,

    // Protocol configuration
    pub _weight_model: WeightModel,
    pub _resolution_type: ResolutionType,
    pub _epoch_duration: i64,
}

#[derive(BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum EpochResultState {
    Active,
    Pending,
    Resolved,
}

#[derive(BorshDeserialize, Debug)]
pub struct EpochAccount {
    pub _epoch: u64,
    pub _weight: u64,
    pub _total_position_amount: u64,
    pub end_at: i64,
    pub _winning_pool_id: u8,
    pub epoch_result_state: EpochResultState,
    pub pool_count: u8,
    pub _pool_weights: [u64; 10 as usize],
}

#[derive(BorshDeserialize, Debug)]
pub struct TaskAccount {
    pub config_pda: Pubkey,
    pub epoch_result_pda: Pubkey,
    pub program_id: Pubkey,
    pub epoch: u64,
    pub end_at: i64,
    pub epoch_result_state: EpochResultState,
    pub pool_count: u8,
    pub custom_pda: Option<Pubkey>,
}