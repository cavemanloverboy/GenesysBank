use anchor_lang::constant;

#[constant]
pub const MAX_RESERVE_TOKEN_AMOUNT: u64 = 10_000_000;
#[constant]
pub const MAX_USER_DEPOSIT: u64 = 100_000;
#[constant]
pub const INTEREST_RATE_TENTHBPS: u64 = 35;
#[constant]
pub const TOKEN_VAULT_SEED: &str = "token-vault";
#[constant]
pub const VAULT_INFO_SEED: &str = "vault-info";
#[constant]
pub const USER_VAULT_SEED: &str = "user-vault";
#[constant]
pub const USER_DEPOSIT_INFO: &str = "user-deposit-info";