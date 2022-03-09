use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod instructions;
pub mod constants;

use instructions::{
    initialize::*,
    deposit::*,
    withdraw::*,
    refresh_reserve::*
};
use crate::constants::*;

#[program]
pub mod genesys_banking {

    use super::*;

    pub fn initialize(
        ctx: Context<InitializeVault>
    ) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn deposit(
        // Boilerplate args
        ctx: Context<Deposit>,
        _reserve_bump: u8,
        _vault_info_bump: u8,
        _deposit_info_bump: u8,
        _user_vault_bump: u8,
        // User-required args
        seconds_locked: u64,
        deposit_lamports: u64,
    ) -> Result<()> {
        instructions::deposit::handler(ctx, seconds_locked, deposit_lamports)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        _reserve_bump: u8,
        vault_info_bump: u8,
        _deposit_info_bump: u8,
        _user_vault_bump: u8,
    ) -> Result<()> {
        instructions::withdraw::handler(ctx, vault_info_bump)
    }

    pub fn refresh_reserve(
        ctx: Context<RefreshReserve>,
        _info_bump: u8,
        reserve_bump: u8,
    ) -> Result<()> {
        instructions::refresh_reserve::handler(ctx, reserve_bump)
    }
}


