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
    pub admin: Pubkey,
    pub resolver: Pubkey,
    pub current_epoch: u64,
    pub total_positions_minted: u64,
    pub position_price: u64,
    pub remaining_total_position: u64,
    pub allowed_mint: Pubkey,
    pub treasury_ata: Pubkey,

    // Protocol configuration
    pub weight_model: WeightModel,
    pub resolution_type: ResolutionType,
    pub epoch_duration: i64,
}

#[derive(BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum EpochResultState {
    Active,
    Pending,
    Resolved,
}

#[derive(BorshDeserialize, Debug)]
pub struct EpochAccount {
    pub epoch: u64,
    pub weight: u64,
    pub total_position_amount: u64,
    pub end_at: i64,
    pub winning_pool_id: u8,
    pub epoch_result_state: EpochResultState,
    pub pool_count: u8,
    pub pool_weights: [u64; 10 as usize],
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