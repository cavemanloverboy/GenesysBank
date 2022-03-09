use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token};
use crate::constants::*;



pub fn handler(
    ctx: Context<InitializeVault>,
    ) -> Result<()> {

    // Grab vault_info from accounts
    let vault_info = &mut ctx.accounts.vault_info;

    // Set reserve maximum and interest rate
    vault_info.max_tokens = MAX_RESERVE_TOKEN_AMOUNT;
    vault_info.interest_rate = INTEREST_RATE_TENTHBPS;

    // Set vault authority
    vault_info.vault_admin = ctx.accounts.vault_admin.key();

    // Set token vault
    vault_info.token_vault = ctx.accounts.token_vault.key();

    // Set token mint
    vault_info.token_mint = ctx.accounts.token_mint.key();

    Ok(())
}

#[derive(Accounts)]
/// This InitializeVault context is used to initialize the bank vault which holds a reserve of some SPL token.
/// 
/// Requirements
/// ----------------------
/// 1) It should hold up to 10,000,000 of a custom SPL token.
/// 2) Only some vault_admin should have the authority to mint + deposit this SPL token.
pub struct InitializeVault<'info> {

    /// This account is a PDA that holds the metadata for the vault
    #[account(
        init,
        payer = vault_admin,
        seeds = [VAULT_INFO_SEED.as_bytes()],
        bump,
    )]
    pub vault_info: Account<'info, VaultInfo>,

    /// This mint account holds the mint info of the SPL token
    #[account(
        init,
        payer = vault_admin,
        mint::decimals = 0,
        mint::authority = vault_admin,
        mint::freeze_authority = vault_admin
    )]
    pub token_mint: Account<'info, Mint>,

    /// This token account is PDA which serves as the reserve for the SPL token
    #[account(
        init,
        payer = vault_admin,
        seeds = [TOKEN_VAULT_SEED.as_bytes()],
        bump,
        token::mint = token_mint,
        token::authority = vault_info,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// This account is the vault admin
    #[account(mut)]
    pub vault_admin: Signer<'info>,

    /// System Program
    pub system_program: Program<'info, System>,
    
    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
}



#[account]
#[derive(Default)]
/// This struct holds all of the metadata for the vault
pub struct VaultInfo {

    /// Maximum number of tokens in vault
    pub max_tokens: u64,

    /// Interest rate (in tenths of bps)
    pub interest_rate: u64,

    /// The vault admin with mint+deposit authority
    pub vault_admin: Pubkey,

    /// The mint of the SPL token stored in the vault
    pub token_mint: Pubkey,

    /// The address of the vault holding the reserve
    pub token_vault: Pubkey,

}