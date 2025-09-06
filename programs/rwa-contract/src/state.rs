use anchor_lang::prelude::*;

#[account]
pub struct GlobalState {
    pub admin: Pubkey,
    pub max_supply: u64,
    pub total_supply: u64,
    pub paused: bool,
}


#[error_code]
pub enum CustomError {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Minting this amount would exceed the maximum supply cap")]
    SupplyExceeded,
    #[msg("Token transfers are currently paused")]
    TransfersPaused,
    #[msg("Math overflow")]
    MathOverflow,
}

#[event]
pub struct MintEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct TransferEvent {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct RedeemEvent {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}


