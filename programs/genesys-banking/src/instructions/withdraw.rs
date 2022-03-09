use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{TokenAccount, Mint, Token}
};
use spl_token;
use solana_program;

use crate::instructions::initialize::VaultInfo;
use crate::instructions::deposit::DepositInfo;
use crate::constants::*;


pub fn handler(
    ctx: Context<Withdraw>,
    vault_info_bump: u8,
) -> Result<()> {

    // Grab deposit info
    let deposit_info = &ctx.accounts.deposit_info;

    // Check if user has waited enough time
    msg!("It's been {} units of time since deposit", deposit_info.get_elapsed());
    require!(deposit_info.after_lockout(), WithdrawError::TooSoon);

    // Check if reserve vault has enough to pay user
    let user_payout = deposit_info.compute_interest();
    require!(ctx.accounts.token_vault.amount >= user_payout, WithdrawError::NotEnoughTokensInReserve);

    require!(*ctx.program_id == ctx.accounts.program.key(), WithdrawError::InvalidProgramId);
    // First, put payout tokens in user vault
    // Construct instruction using spl_token library
    msg!("transferring from reserve to user vault");
    let ix = spl_token::instruction::transfer_checked(

        // token_program_id: &Pubkey, 
        // source_pubkey: &Pubkey, 
        // mint_pubkey: &Pubkey, 
        // destination_pubkey: &Pubkey, 
        // authority_pubkey: &Pubkey, 
        // signer_pubkeys: &[&Pubkey], 
        // amount: u64, 
        // decimals: u8

        &ctx.accounts.token_program.key(),
        &ctx.accounts.token_vault.key(),
        &ctx.accounts.token_mint.key(),
        &ctx.accounts.user_vault.key(),
        &ctx.accounts.vault_info.key(),
        &[&ctx.accounts.vault_info.key()],
        user_payout,
        ctx.accounts.token_mint.decimals,
    )?;

    msg!("invoking for transfer from reserve to user vault");
    // Invoke using solana_program library
    solana_program::program::invoke_signed(
        &ix,
        &[
            //ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_vault.to_account_info(),
            ctx.accounts.vault_info.to_account_info(),
            ctx.accounts.token_mint.to_account_info(),
            ctx.accounts.user_vault.to_account_info(),
            ctx.accounts.vault_admin.to_account_info(),
        ],
        &[&[VAULT_INFO_SEED.as_bytes(), &[vault_info_bump]]]
    ).expect("failed invoking spl transfer");

    msg!("transferring from user vault to user");
    // Second, return all tokens in user vault to user
    let ix = spl_token::instruction::transfer_checked(

        // token_program_id: &Pubkey, 
        // source_pubkey: &Pubkey, 
        // mint_pubkey: &Pubkey, 
        // destination_pubkey: &Pubkey, 
        // authority_pubkey: &Pubkey, 
        // signer_pubkeys: &[&Pubkey], 
        // amount: u64, 
        // decimals: u8

        &ctx.accounts.token_program.key(),
        &ctx.accounts.user_vault.key(),
        &ctx.accounts.token_mint.key(),
        &ctx.accounts.depositor_token_account.key(),
        &ctx.accounts.vault_info.key(),
        &[&ctx.accounts.vault_info.key()],
        deposit_info.deposit_lamports + user_payout,
        ctx.accounts.token_mint.decimals,
    )?;

    msg!("invoking for transfer from user vault to user");
    // Invoke using solana_program library
    solana_program::program::invoke_signed(
        &ix,
        &[
            //ctx.accounts.token_program.to_account_info(),
            ctx.accounts.user_vault.to_account_info(),
            ctx.accounts.vault_info.to_account_info(),
            ctx.accounts.token_mint.to_account_info(),
            ctx.accounts.depositor_token_account.to_account_info(),
            ctx.accounts.vault_admin.to_account_info(),
        ],
        &[&[VAULT_INFO_SEED.as_bytes(), &[vault_info_bump]]]
    )?;


    Ok(())
}



#[derive(Accounts)]
#[instruction(
    reserve_bump: u8,
    vault_info_bump: u8,
    deposit_info_bump: u8,
    user_vault_bump: u8,
)]
/// This InitializeVault context is used to initialize the bank vault which holds a reserve of some SPL token.
/// 
/// Requirements
/// ----------------------
/// 1) It should hold up to 10,000,000 of a custom SPL token.
/// 2) Only some admin should have the authority to mint + deposit this SPL token.
pub struct Withdraw<'info> {

    /// This account holds the metadata for the vault
    #[account(
        seeds = [
            USER_DEPOSIT_INFO.as_bytes(), 
            &depositor.key.to_bytes(),
        ],
        bump = deposit_info_bump,
    )]
    pub deposit_info: Box<Account<'info, DepositInfo>>,

    /// This account holds the metadata for the vault
    #[account(
        seeds = [VAULT_INFO_SEED.as_bytes()],
        bump = vault_info_bump
    )]
    pub vault_info: Box<Account<'info, VaultInfo>>,

    /// This token account serves as the account which holds the SPL token
    #[account(
        mut,
        seeds = [
            USER_VAULT_SEED.as_bytes(),
            &depositor.key.to_bytes(),
        ],
        bump = user_vault_bump,
    )]
    pub user_vault: Box<Account<'info, TokenAccount>>,

    /// This token account is PDA which serves as the reserve for the SPL token
    #[account(
        mut, 
        address=vault_info.token_vault
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    /// This mint account holds the mint info of the SPL token
    #[account(address=vault_info.token_mint)]
    pub token_mint: Box<Account<'info, Mint>>,

    /// This is the vault admin
    /// CHECK: This is fine because we are ensuring address=vault_info.admin
    #[account(address=vault_info.vault_admin)]
    pub vault_admin: AccountInfo<'info>,

    /// This account is the user/depositor
    #[account(mut)]
    pub depositor: Signer<'info>,

    /// This account is the user's SPL token account
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = token_mint,
        associated_token::authority = depositor,
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    /// System Program
    pub system_program: Program<'info, System>,
    
    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Token Program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,

    /// Our Program
    /// CHECK: This is fine because we are validating this is indeed our program
    pub program: AccountInfo<'info>,
}


#[error_code]
pub enum WithdrawError {
    #[msg("User is trying to withdraw too soon")]
    TooSoon,
    #[msg("The reserve does not have enough tokens to pay you right now")]
    NotEnoughTokensInReserve,
    #[msg("Passed in wrong program_id")]
    InvalidProgramId
}