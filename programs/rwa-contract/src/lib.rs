use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{spl_token::native_mint::DECIMALS, Mint, Token, TokenAccount}};

declare_id!("A5yP5JM5ihdshwUtqGFizLKZzar846xzY5wWaAyHwymA");

#[program]
pub mod rwa_contract {
    use anchor_spl::token::{self, Burn, InitializeAccount, InitializeMint, MintTo, Transfer};

    use super::*;

    pub fn initialize_mint(ctx: Context<Initialize>, decimals: u8) -> Result<()> {
        let bump = ctx.bumps.mint_authority;
        let authority_seeds: &[&[u8]] = &[b"mint_authority".as_ref(), &[bump]];
        
        token::initialize_mint(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::InitializeMint {
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &[&[b"mint_authority".as_ref(), &[bump]]],
            ), 
            decimals, 
            &ctx.accounts.mint_authority.key(), 
            Some(&ctx.accounts.mint_authority.key()),
        )?;
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
            state.total_supply + amount <= state.max_supply,
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
                    authority: ctx.accounts.mint_authority.to_account_info()
                },
                signer_seeds,
            ),
            amount,
        )?;

        state.total_supply += amount;

        emit!(MintEvent {
            recipient: ctx.accounts.recipient_token_account.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp
        });

        Ok(())
    }

    pub fn transfer_token(ctx: Context<TransferTokenCtx>, amount: u64) -> Result<()> {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info()
                }
            ), 
            amount,
        )
    }

    pub fn burn_token(ctx: Context<BurnTokenCtx>, amount: u64) -> Result<()> {
        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                Burn {
                    mint: ctx.accounts.mint.to_account_info(),
                    from: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info()
                }
            ), 
            amount,
        )
    }
}


// ---------- Contexts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        mint::decimals = DECIMALS,
        mint::authority = mint_authority, 
        mint::freeze_authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]

    pub mint_authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
  #[account(mut)]
  pub admin: Signer<'info>,

  #[account(mut)]
  pub mint: Account<'info, Mint>,

  #[account(mut)]
  pub recipient_token_account: Account<'info, TokenAccount>,

  #[account(
    seeds = [b"mint_authority"],
    bump
  )]

  pub mint_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"global-state"],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,

  pub token_program: Program<'info, Token>
}


//////

#[derive(Accounts)]
pub struct TransferTokenCtx<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct BurnTokenCtx<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>
}


/////
#[account]
pub struct GlobalState {
    pub admin: Pubkey,
    pub max_supply: u64,
    pub total_supply: u64
}

#[derive(Accounts)]
pub struct InitializeGlobalState<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 8 + 8, // discriminator + pubkey + 2 u64s
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum CustomError {
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Minting this amount would exceed the maximum supply cap")]
    SupplyExceeded,
}

#[event]
pub struct MintEvent {
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

