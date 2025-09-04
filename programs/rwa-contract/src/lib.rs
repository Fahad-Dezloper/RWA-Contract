use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, Transfer};

mod state;
mod contexts;

pub use contexts::*;
pub use state::*;

declare_id!("A5yP5JM5ihdshwUtqGFizLKZzar846xzY5wWaAyHwymA");


#[program]
pub mod rwa_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Mint initialized: {}", ctx.accounts.mint.key());
        Ok(())
    }

    pub fn initialize_global_state(ctx: Context<InitializeGlobalState>, max_supply: u64) -> Result<()> {
        let state = &mut ctx.accounts.global_state;
        state.admin = ctx.accounts.admin.key();
        state.max_supply = max_supply;
        state.total_supply = 0;
        state.paused = false;
        Ok(())
    }

    pub fn mint_rwa_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.admin.key(),
            ctx.accounts.global_state.admin,
            CustomError::Unauthorized
        );

        let state = &mut ctx.accounts.global_state;
        require!(
            state
                .total_supply
                .checked_add(amount)
                .ok_or(CustomError::MathOverflow)?
                <= state.max_supply,
            CustomError::SupplyExceeded
        );

        let bump = ctx.bumps.mint_authority;
        let seeds: &[&[u8]] = &[b"mint_authority".as_ref(), &[bump]];
        let signer_seeds = &[&seeds[..]];

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.recipient_token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
        )?;

        state.total_supply = state
            .total_supply
            .checked_add(amount)
            .ok_or(CustomError::MathOverflow)?;
        emit!(MintEvent {
            recipient: ctx.accounts.recipient_token_account.owner,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn transfer_rwa_token(ctx: Context<TransferTokenCtx>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.global_state;
        require!(!state.paused, CustomError::TransfersPaused);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(TransferEvent {
            from: ctx.accounts.from.owner,
            to: ctx.accounts.to.owner,
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn burn_token(ctx: Context<BurnTokenCtx>, amount: u64) -> Result<()> {
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }

    pub fn create_escrow(
        ctx: Context<CreateEscrow>,
        beneficiary: Pubkey,
        amount: u64,
    ) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.depositor = ctx.accounts.depositor.key();
        escrow.beneficiary = beneficiary;
        escrow.amount = amount;
        escrow.state = EscrowState::Active;
        escrow.bump = ctx.bumps.escrow_account;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.depositor_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.depositor.to_account_info(),
                },
            ),
            amount,
        )?;

        emit!(EscrowCreatedEvent {
            depositor: escrow.depositor,
            beneficiary,
            amount,
        });

        Ok(())
    }

    pub fn release_escrow(ctx: Context<ReleaseEscrow>) -> Result<()> {
        {
            let escrow_check = &ctx.accounts.escrow_account;
            require!(escrow_check.state == EscrowState::Active, CustomError::EscrowNotActive);
        }

        let escrow_info = ctx.accounts.escrow_account.to_account_info();
        let escrow = &mut ctx.accounts.escrow_account;

        let seeds: &[&[u8]] = &[
            b"escrow",
            escrow.depositor.as_ref(),
            escrow.beneficiary.as_ref(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.beneficiary_token_account.to_account_info(),
                    authority: escrow_info,
                },
                signer_seeds,
            ),
            escrow.amount,
        )?;

        escrow.state = EscrowState::Released;

        emit!(EscrowReleasedEvent {
            depositor: escrow.depositor,
            beneficiary: escrow.beneficiary,
            amount: escrow.amount,
        });

        Ok(())
    }

    pub fn cancel_escrow(ctx: Context<CancelEscrow>) -> Result<()> {
        {
            let escrow_check = &ctx.accounts.escrow_account;
            require!(escrow_check.state == EscrowState::Active, CustomError::EscrowNotActive);
        }

        let escrow_info = ctx.accounts.escrow_account.to_account_info();
        let escrow = &mut ctx.accounts.escrow_account;

        let seeds: &[&[u8]] = &[
            b"escrow",
            escrow.depositor.as_ref(),
            escrow.beneficiary.as_ref(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.depositor_token_account.to_account_info(),
                    authority: escrow_info,
                },
                signer_seeds,
            ),
            escrow.amount,
        )?;

        escrow.state = EscrowState::Cancelled;

        emit!(EscrowCancelledEvent {
            depositor: escrow.depositor,
            beneficiary: escrow.beneficiary,
            amount: escrow.amount,
        });

        Ok(())
    }
}
