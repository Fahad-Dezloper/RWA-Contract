use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::{EscrowAccount, GlobalState};

pub const TOKEN_DECIMALS: u8 = 6;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority
    )]
    pub mint: Account<'info, Mint>,
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8 + 8 + 1,
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"global-state"], bump)]
    pub global_state: Account<'info, GlobalState>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferTokenCtx<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    #[account(seeds = [b"global-state"], bump)]
    pub global_state: Account<'info, GlobalState>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokenCtx<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(beneficiary: Pubkey)]
pub struct CreateEscrow<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = depositor,
        space = 8 + 32 + 32 + 8 + 1 + 1,
        seeds = [b"escrow", depositor.key().as_ref(), beneficiary.as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = depositor,
        token::mint = mint,
        token::authority = escrow_account,
        seeds = [b"vault", depositor.key().as_ref(), beneficiary.as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ReleaseEscrow<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        seeds = [b"escrow", escrow_account.depositor.as_ref(), escrow_account.beneficiary.as_ref()],
        bump = escrow_account.bump,
        has_one = depositor,
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut, seeds = [b"vault", escrow_account.depositor.as_ref(), escrow_account.beneficiary.as_ref()], bump)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub beneficiary_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelEscrow<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        seeds = [b"escrow", escrow_account.depositor.as_ref(), escrow_account.beneficiary.as_ref()],
        bump = escrow_account.bump,
        has_one = depositor,
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    #[account(mut, seeds = [b"vault", escrow_account.depositor.as_ref(), escrow_account.beneficiary.as_ref()], bump)]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub depositor_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}


