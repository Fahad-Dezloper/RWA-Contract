use anchor_lang::prelude::*;
use anchor_spl::{token::{Mint, Token, TokenAccount}};

declare_id!("A5yP5JM5ihdshwUtqGFizLKZzar846xzY5wWaAyHwymA");

#[program]
pub mod rwa_contract {
    use anchor_spl::{token::{self, Burn, InitializeAccount, InitializeMint, MintTo, Transfer}};

    use super::*;

    pub fn initialize_mint(ctx: Context<Initialize>, decimals: u8) -> Result<()> {
        token::initialize_mint(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                InitializeMint{
                    mint: ctx.accounts.mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }
            ),
            decimals, 
            ctx.accounts.authority.key, 
            Some(ctx.accounts.authority.key)
        )?;
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn initialize_token_account(ctx: Context<InitializeMintToken>) -> Result<()> {
        token::initialize_account(
            CpiContext::new(
                ctx.accounts.token_account.to_account_info(), 
                InitializeAccount {
                    account: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }
            )
        )?;
        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_account.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info()
                }
            ),
            amount
        )?;
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
            amount
        )?;
        Ok(())
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
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 82,
        seeds = [b"mint"],
        bump
    )]

    pub mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct InitializeMintToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = 165,
    )]

    pub token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct TransferTokenCtx<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct BurnTokenCtx<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>
}

