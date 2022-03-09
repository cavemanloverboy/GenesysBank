use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{TokenAccount, Mint, Token}
};
use spl_token;
use solana_program;

use crate::instructions::initialize::VaultInfo;
use crate::constants::*;


pub fn handler(
    ctx: Context<Deposit>,
    seconds_locked: u64,
    deposit_lamports: u64,
    ) -> Result<()> {

    // Ensure user is not depositing more than is allowed
    require!(deposit_lamports <= MAX_USER_DEPOSIT, DepositError::MaxDepositLimit);

    // Ensure user is depositiing for nonzero time
    require!(seconds_locked > 0, DepositError::ZeroTimeDeposit);

    // Ensure user is depositing for less than what would break our setup
    // i.e. interest owed > max tokens in vault
    require!(seconds_locked < max_time(deposit_lamports), DepositError::BreakingTheBank);

    // Initialize deposit_info account data
    let deposit_info = &mut ctx.accounts.deposit_info;
    deposit_info.seconds_locked = seconds_locked;
    deposit_info.deposit_lamports = deposit_lamports;
    deposit_info.depositor = ctx.accounts.depositor.key();
    deposit_info.deposit_time = Clock::get().unwrap().unix_timestamp;

    // Construct instruction using spl_token library
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
        &ctx.accounts.depositor_token_account.key(),
        &ctx.accounts.token_mint.key(),
        &ctx.accounts.user_vault.key(),
        &ctx.accounts.depositor.key(),//&ctx.accounts.vault_admin.key(),
        &[&ctx.accounts.depositor.key()],
        deposit_lamports,
        ctx.accounts.token_mint.decimals,
    )?;

    // Invoke using solana_program library
    solana_program::program::invoke(
        &ix,
        &[
            //ctx.accounts.token_program.to_account_info(),
            ctx.accounts.depositor_token_account.to_account_info(),
            ctx.accounts.depositor.to_account_info(),
            ctx.accounts.token_mint.to_account_info(),
            ctx.accounts.user_vault.to_account_info(),
            ctx.accounts.vault_admin.to_account_info(),
        ],
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
pub struct Deposit<'info> {

    /// This account holds the metadata for the vault
    #[account(
        init,
        payer = depositor,
        seeds = [
            USER_DEPOSIT_INFO.as_bytes(), 
            &depositor.key.to_bytes(),
        ],
        bump,
    )]
    pub deposit_info: Account<'info, DepositInfo>,

    /// This account holds the metadata for the vault
    #[account(
        seeds = [VAULT_INFO_SEED.as_bytes()],
        bump = vault_info_bump
    )]
    pub vault_info: Account<'info, VaultInfo>,

    /// This token account serves as the account which holds the SPL token
    #[account(
        init,
        payer = depositor,
        seeds = [
            USER_VAULT_SEED.as_bytes(),
            &depositor.key.to_bytes(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = vault_info,
    )]
    pub user_vault: Account<'info, TokenAccount>,

    /// This mint account holds the mint info of the SPL token
    #[account(address=vault_info.token_mint)]
    pub token_mint: Account<'info, Mint>,

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
}



#[account]
#[derive(Default)]
/// This struct holds all of the metadata for the vault
pub struct DepositInfo {

    /// Time in seconds deposited
    pub seconds_locked: u64,

    /// Deposited amount
    pub deposit_lamports: u64,

    /// The depositor
    pub depositor: Pubkey,

    /// Time deposited
    pub deposit_time: i64,
    
}

impl DepositInfo{

    pub fn get_elapsed(&self) -> i64 {
        Clock::get().unwrap().unix_timestamp - self.deposit_time
    }

    pub fn after_lockout(&self) -> bool {
        self.get_elapsed() as u64 >= self.seconds_locked
    }
    
    pub fn compute_interest(&self) -> u64 {
        (self.deposit_lamports as f64 
            * ((1.0 + INTEREST_RATE_TENTHBPS as f64/100000.0).powf(self.seconds_locked as f64) - 1.0))
             as u64
    }
}


#[error_code]
pub enum DepositError {
    #[msg(format!("Attempting to deposit over limit of {} tokens", MAX_USER_DEPOSIT))]
    MaxDepositLimit,
    #[msg("Attempting to deposit for zero time")]
    ZeroTimeDeposit,
    #[msg("Attempting to deposit for an amount of time that would break the bank")]
    BreakingTheBank,
}

fn max_time(
    deposit_lamports: u64,
) -> u64 {
    // this solves deposit_lamports * (1 + interest)^seconds = max_vault_balance
    ((MAX_RESERVE_TOKEN_AMOUNT as f64/deposit_lamports as f64).log(10.0)
    /(1.0 +INTEREST_RATE_TENTHBPS as f64/ 100000.0).log(10.0)) as u64
}

#[test]
fn test_max_time(){
    assert_eq!(max_time(100_000), 13_159)
}