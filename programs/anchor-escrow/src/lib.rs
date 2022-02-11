use anchor_lang::prelude::*;
use anchor_spl::token::{self, SetAuthority, Token, TokenAccount};
use spl_token::instruction::AuthorityType;

declare_id!("Ei6ZoGTRyYMcm4aWLfvdFCUUw76qtPLgJ9QkSpP9hHAe");

#[program]
pub mod escrow {
    use super::*;

    const ESCROW_PDA_SEED: &[u8] = b"escrow";

    pub fn list(
        ctx: Context<List>,
        initializer_amount: u64,
    ) -> ProgramResult {
        ctx.accounts.escrow_account.seller = *ctx.accounts.initializer.key;
        ctx.accounts.escrow_account.token_account_pubkey = 
        *ctx.accounts.initializer_token_account.to_account_info().key;
        ctx.accounts.escrow_account.amount = initializer_amount;
        
        let escrow_key = ctx.accounts.escrow_account.key();
        let (pda, _bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED, escrow_key.as_ref()], ctx.program_id);
        token::set_authority(ctx.accounts.into(), AuthorityType::AccountOwner, Some(pda))?;
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> ProgramResult {
        let escrow_key = ctx.accounts.escrow_account.key();
        let (_pda, bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED, escrow_key.as_ref()], ctx.program_id);
        let seeds = &[&ESCROW_PDA_SEED, escrow_key.as_ref(), &[bump_seed]];

        token::set_authority(
            ctx.accounts.into_set_authority_context().with_signer(&[&seeds[..]]),
            AuthorityType::AccountOwner,
            Some(ctx.accounts.escrow_account.seller),
        )?;

        Ok(())
    }

    pub fn buy(ctx: Context<Buy>) -> ProgramResult {
        // Transferring from initializer to taker
        let escrow_key = ctx.accounts.escrow_account.key();
        let (_pda, bump_seed) = Pubkey::find_program_address(&[ESCROW_PDA_SEED, escrow_key.as_ref()], ctx.program_id);
        let seeds = &[&ESCROW_PDA_SEED, escrow_key.as_ref(), &[bump_seed]];

        let system_program = ctx.accounts.token_program.to_account_info();

        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(ctx.accounts.buyer.key, ctx.accounts.initializer_main_account.key, ctx.accounts.escrow_account.amount),
            &[
                ctx.accounts.buyer.clone(),
                ctx.accounts.initializer_main_account.clone(),
                system_program.to_account_info().clone(),
            ],
        )?;

        token::set_authority(
            ctx.accounts
                .into_set_authority_context()
                .with_signer(&[&seeds[..]]),
            AuthorityType::AccountOwner,
            Some(ctx.accounts.escrow_account.seller),
        )?;

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(initializer_amount: u64)]
pub struct List<'info> {
    #[account(signer, mut)]
    pub initializer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = initializer_token_account.amount >= initializer_amount
    )]
    pub initializer_token_account: Account<'info, TokenAccount>,
    #[account(init, payer = initializer, space = 8 + EscrowAccount::LEN)]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(signer, mut)]
    pub buyer: AccountInfo<'info>,
    #[account(mut)]
    pub pda_deposit_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub initializer_main_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.token_account_pubkey == *pda_deposit_token_account.to_account_info().key,
        constraint = escrow_account.seller == *initializer_main_account.key,
        close = initializer_main_account
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub pda_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub user: AccountInfo<'info>,
    #[account(mut)]
    pub pda_token_account: Account<'info, TokenAccount>,
    pub pda_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint = escrow_account.seller == *user.key,
        constraint = escrow_account.token_account_pubkey == *pda_token_account.to_account_info().key,
        close = user
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct EscrowAccount {
    pub is_initialized: bool,
    pub token_account_pubkey: Pubkey,
    pub mint_key: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
}

impl EscrowAccount {
    pub const LEN: usize = 1 + 32 + 32 + 32 + 8;
}

impl<'info> From<&mut List<'info>>
    for CpiContext<'_, '_, '_, 'info, SetAuthority<'info>>
{
    fn from(accounts: &mut List<'info>) -> Self {
        let cpi_accounts = SetAuthority {
            account_or_mint: accounts
                .initializer_token_account
                .to_account_info()
                .clone(),
            current_authority: accounts.initializer.clone(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> Cancel<'info> {
    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.pda_token_account.to_account_info().clone(),
            current_authority: self.pda_account.clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'info> Buy<'info> {
    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.pda_deposit_token_account.to_account_info().clone(),
            current_authority: self.pda_account.clone(),
        };
        let cpi_program = self.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}