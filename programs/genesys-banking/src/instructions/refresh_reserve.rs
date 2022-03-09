use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token};
use spl_token;
use solana_program;

use crate::instructions::initialize::VaultInfo;
use crate::constants::*;


pub fn handler(
    ctx: Context<RefreshReserve>,
    reserve_bump: u8,
    ) -> Result<()> {

    // Compute refresh amount
    let refresh_amount = MAX_RESERVE_TOKEN_AMOUNT.checked_sub(ctx.accounts.token_vault.amount);
    require!(refresh_amount.is_some(), RefreshError::MaxReserveLimit);

    require!(*ctx.program_id == ctx.accounts.program.key(), RefreshError::InvalidProgramId);

    msg!("Creating mint ix");
    // Construct instruction using spl_token library
    let ix = spl_token::instruction::mint_to(

        // token_program_id: &Pubkey, 
        // mint_pubkey: &Pubkey, 
        // account_pubkey: &Pubkey, 
        // owner_pubkey: &Pubkey, 
        // signer_pubkeys: &[&Pubkey], 
        // amount: u64
        // decimals: u8

        &ctx.accounts.token_program.key(),
        &ctx.accounts.token_mint.key(),
        &ctx.accounts.token_vault.key(),
        &ctx.accounts.vault_admin.key(),//&ctx.program_id,
        &[],//&ctx.accounts.vault_admin.key()],
        refresh_amount.unwrap(),
        //ctx.accounts.token_mint.decimals,
    )?;

    msg!("mint ix invoke");
    msg!("token program: {}", ctx.accounts.token_program.key());
    msg!("mint key: {}", ctx.accounts.token_mint.key());
    msg!("program id: {}", ctx.accounts.program.key());
    msg!("vault info: {}", ctx.accounts.vault_info.key());
    msg!("vault key: {}", ctx.accounts.token_vault.key());
    msg!("admin key: {}", ctx.accounts.vault_admin.key());
    // Invoke using solana_program library
    let seeds: &[&[u8]] = &[TOKEN_VAULT_SEED.as_bytes()];
    solana_program::program::invoke_signed(
        &ix,
        &[
            //ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_mint.to_account_info(),
            //ctx.accounts.program.to_account_info(),
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.vault_admin.to_account_info(),
        ],
        //&[&[TOKEN_VAULT_SEED.as_bytes()]],
        &[&[seeds, &[&[reserve_bump]]].concat()],
    ).expect("failed to refresh reserve");

    Ok(())

    // mint_to(
    //     &ctx.accounts.token_vault.to_account_info(), 
    //     &ctx.accounts.token_mint.to_account_info(),
    //     &ctx.accounts.vault_admin.to_account_info(),
    //     &ctx.accounts.program.key(),//base_address: &Pubkey, 
    //     &[TOKEN_VAULT_SEED.as_bytes()], refresh_amount.unwrap() //seeds: &[&[u8]], amount: u64)
    // )
}

#[derive(Accounts)]
#[instruction(
    info_bump: u8,
    reserve_bump: u8,
)]
/// This InitializeVault context is used to initialize the bank vault which holds a reserve of some SPL token.
/// 
/// Requirements
/// ----------------------
/// 1) It should hold up to 10,000,000 of a custom SPL token.
/// 2) Only some admin should have the authority to mint + deposit this SPL token.
pub struct RefreshReserve<'info> {

    /// This account holds the metadata for the vault
    #[account(
        seeds = [VAULT_INFO_SEED.as_bytes()],
        bump = info_bump,
    )]
    pub vault_info: Account<'info, VaultInfo>,

    /// This token account serves as the reserve for the SPL token
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED.as_bytes()],
        bump = reserve_bump,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// This mint account holds the mint info of the SPL token
    #[account(mut, address=vault_info.token_mint)]
    pub token_mint: Account<'info, Mint>,

    /// This is the vault admin
    /// CHECK: This is fine because we are ensuring address=vault_info.admin
    #[account(address=vault_info.vault_admin)]
    pub vault_admin: Signer<'info>,

    /// System Program
    pub system_program: Program<'info, System>,
    
    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
    
    /// Our Program
    /// CHECK: this is fine because we are ensuring this is our program
    #[account()]
    pub program: AccountInfo<'info>,
}



#[account]
#[derive(Default)]
/// This struct holds all of the metadata for the vault
pub struct DepositInfo {

    /// Time in seconds deposited
    pub seconds_locked: u64,

    /// Deposited amount
    pub deposit_amount: u64,

    /// The depositor
    pub depositor: Pubkey,
    
}


#[error_code]
pub enum RefreshError {
    #[msg("Attempting to refresh when reserve is full")]
    MaxReserveLimit,
    #[msg("Passed in wrong program_id")]
    InvalidProgramId,
}