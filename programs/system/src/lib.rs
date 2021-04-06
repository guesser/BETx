#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, TokenAccount, Transfer};

#[program]
mod system {

    use super::*;
    #[state]
    pub struct InternalState {
        pub nonce: u8,
        pub signer: Pubkey,
        pub oracle: Pubkey,
        pub mint_authority: Pubkey,
        pub collateral_token: Pubkey,
        pub collateral_minted: u64,
        pub vault: Pubkey,
        pub winner: Pubkey,
        pub expiration_time: i64,
        pub outcome1: Outcome,
        pub outcome2: Outcome,
    }

    impl InternalState {
        pub fn new(_ctx: Context<New>) -> Result<Self> {
            let outcome1 = Outcome {
                address: Pubkey::default(),
                ticker: "YES".as_bytes().to_vec(),
                decimals: 8,
            };
            let outcome2 = Outcome {
                address: Pubkey::default(),
                ticker: "NO".as_bytes().to_vec(),
                decimals: 8,
            };
            Ok(Self {
                nonce: 0,
                signer: Pubkey::default(),
                oracle: Pubkey::default(),
                mint_authority: Pubkey::default(),
                collateral_token: Pubkey::default(),
                collateral_minted: 0,
                vault: Pubkey::default(),
                winner: Pubkey::default(),
                expiration_time: 0,
                outcome1,
                outcome2,
            })
        }
        pub fn initialize(
            &mut self,
            _ctx: Context<Initialize>,
            nonce: u8,
            signer: Pubkey,
            oracle: Pubkey,
            collateral_token: Pubkey,
            vault: Pubkey,
            mint_authority: Pubkey,
            expiration_time: i64,
            outcome1: Pubkey,
            outcome2: Pubkey,
        ) -> Result<()> {
            if self.expiration_time != 0 || self.oracle != Pubkey::default() {
                return Err(ErrorCode::ProgramInitialized.into());
            }
            self.signer = signer;
            self.nonce = nonce;
            self.oracle = oracle;
            self.collateral_token = collateral_token;
            self.vault = vault;
            self.mint_authority = mint_authority;
            self.expiration_time = expiration_time;
            self.outcome1.address = outcome1;
            self.outcome2.address = outcome2;

            Ok(())
        }

        pub fn mint_complete_sets(&mut self, ctx: Context<Mint>, amount: u64) -> Result<()> {
            let deposited = ctx.accounts.collateral_account.amount - self.collateral_minted;
            if deposited == 0 {
                return Err(ErrorCode::ZeroDeposit.into());
            }
            if deposited != amount {
                return Err(ErrorCode::DespositedMismatch.into());
            }
            // TODO: Check the outcomes passed are the ones stored

            let seeds = &[self.signer.as_ref(), &[self.nonce]];
            let signer = &[&seeds[..]];


            // Outcome 1
            let cpi_accounts1 = MintTo {
                mint: ctx.accounts.outcome1.to_account_info(),
                to: ctx.accounts.to1.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_program1 = ctx.accounts.token_program.to_account_info();
            let cpi_context1 = CpiContext::new(cpi_program1, cpi_accounts1).with_signer(signer);
            token::mint_to(cpi_context1, amount)?;

            // Outcome 2
            let cpi_accounts2 = MintTo {
                mint: ctx.accounts.outcome2.to_account_info(),
                to: ctx.accounts.to2.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_program2 = ctx.accounts.token_program.to_account_info();
            let cpi_context2 = CpiContext::new(cpi_program2, cpi_accounts2).with_signer(signer);
            token::mint_to(cpi_context2, amount)?;

            self.collateral_minted = ctx.accounts.collateral_account.amount;

            Ok(())
        }

        pub fn redeem_complete_sets(
            &mut self,
            ctx: Context<RedeemCompleteSets>,
            amount: u64,
        ) -> Result<()> {
            // TODO: Check the outcomes are the ones stored
            // TODO: Check if amount is higher than their amount
            
            let seeds = &[self.signer.as_ref(), &[self.nonce]];
            let signer = &[&seeds[..]];

            // Outcome1
            {
                let cpi_accounts = Burn {
                    mint: ctx.accounts.outcome1.to_account_info(),
                    to: ctx.accounts.to1.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer);
                token::burn(cpi_ctx, amount)?;
            }
            // Outcome2
            {
                let cpi_accounts = Burn {
                    mint: ctx.accounts.outcome2.to_account_info(),
                    to: ctx.accounts.to2.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer);
                token::burn(cpi_ctx, amount)?;
            }

            let cpi_accounts = Transfer {
                from: ctx.accounts.collateral_account.to_account_info().clone(),
                to: ctx.accounts.to.to_account_info().clone(),
                authority: ctx.accounts.authority.to_account_info().clone(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer);
            token::transfer(cpi_ctx, amount)?;

            Ok(())
        }

        pub fn resolve_market(&mut self, ctx: Context<FinishMarket>) -> Result<()> {
            if ctx.accounts.clock.unix_timestamp < self.expiration_time {
                return Err(ErrorCode::ExpirationTimeNotPassed.into());
            }
            if ctx.accounts.oracle.key != &self.oracle {
                return Err(ErrorCode::OraclesMismatch.into());
            }
            if self.winner != Pubkey::default() {
                return Err(ErrorCode::WinnerAlreadySet.into());
            }

            self.winner = *ctx.accounts.winner.key;

            Ok(())
        }

        pub fn claim_profits(&mut self, ctx: Context<ClaimProfits>, amount: u64) -> Result<()> {
            if self.winner == Pubkey::default() {
                return Err(ErrorCode::MarketNotSettled.into());
            }
            if self.winner != *ctx.accounts.winner_outcome.key {
                return Err(ErrorCode::WinnerDoesNotMatch.into());
            }
            if amount == 0 {
                return Err(ErrorCode::NoProfits.into());
            }

            let seeds = &[self.signer.as_ref(), &[self.nonce]];
            let signer = &[&seeds[..]];

            {
                let cpi_accounts = Burn {
                    mint: ctx.accounts.winner_outcome.to_account_info(),
                    to: ctx.accounts.winner_from.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer);
                token::burn(cpi_ctx, amount)?;
            }

            let cpi_accounts = Transfer {
                from: ctx.accounts.collateral_account.to_account_info().clone(),
                to: ctx.accounts.to.to_account_info().clone(),
                authority: ctx.accounts.authority.to_account_info().clone(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer);
            token::transfer(cpi_ctx, amount)?;

            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct New {}
#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Mint<'info> {
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub to1: AccountInfo<'info>,
    #[account(mut)]
    pub to2: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    // pub clock: Sysvar<'info, Clock>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    pub collateral_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub outcome2: AccountInfo<'info>,
    #[account(mut)]
    pub outcome1: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct RedeemCompleteSets<'info> {
    // TODO: Refactor the naming of these words
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    #[account(mut)]
    pub to1: AccountInfo<'info>,
    #[account(mut)]
    pub to2: AccountInfo<'info>,
    #[account(mut)]
    pub token_program: AccountInfo<'info>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    #[account(mut)]
    pub collateral_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub outcome1: AccountInfo<'info>,
    #[account(mut)]
    pub outcome2: AccountInfo<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Default, Clone)]
pub struct Outcome {
    pub address: Pubkey,
    pub decimals: u8,
    pub ticker: Vec<u8>,
}

#[derive(Accounts)]
pub struct ClaimProfits<'info> {
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub winner_from: AccountInfo<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    #[account(mut)]
    pub token_program: AccountInfo<'info>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    #[account(mut)]
    pub collateral_account: CpiAccount<'info, TokenAccount>,
    #[account(mut)]
    pub winner_outcome: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FinishMarket<'info> {
    #[account(signer)]
    oracle: AccountInfo<'info>,
    winner: AccountInfo<'info>,
    clock: Sysvar<'info, Clock>,
}

#[error]
pub enum ErrorCode {
    #[msg("Program already Initialized")]
    ProgramInitialized,
    #[msg("Mint limit crossed")]
    MintLimit,
    #[msg("Wrong token not sythetic usd")]
    NotSyntheticUsd,
    #[msg("Deposited zero")]
    ZeroDeposit,
    #[msg("Expiration time not passed yet")]
    ExpirationTimeNotPassed,
    #[msg("Oracles don't match")]
    OraclesMismatch,
    #[msg("Winner was already set")]
    WinnerAlreadySet,
    #[msg("Desposited mismatches the amount parameter")]
    DespositedMismatch,
    #[msg("Market has not been settled yet")]
    MarketNotSettled,
    #[msg("No profits redemable")]
    NoProfits,
    #[msg("Winner sent is not the correct one")]
    WinnerDoesNotMatch,
}
