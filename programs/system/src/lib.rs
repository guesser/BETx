#![feature(proc_macro_hygiene)]

use anchor_lang::prelude::*;
use anchor_spl::token::{
    self,
    MintTo,
    TokenAccount,
    //Transfer
};

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
        pub vault: Pubkey,
        pub outcomes: Vec<Outcome>,
        pub winner: Pubkey,
        pub expiration_time: u64,
    }

    impl InternalState {
        pub const ASSETS_SIZE: usize = 10;
        pub fn new(_ctx: Context<New>) -> Result<Self> {
            let mut outcomes: Vec<Outcome> = vec![];
            outcomes.resize(
                Self::ASSETS_SIZE,
                Outcome {
                    // ticker: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                    ..Default::default()
                },
            );
            Ok(Self {
                nonce: 0,
                signer: Pubkey::default(),
                oracle: Pubkey::default(),
                mint_authority: Pubkey::default(),
                collateral_token: Pubkey::default(),
                vault: Pubkey::default(),
                winner: Pubkey::default(),
                expiration_time: 0,
                outcomes,
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
            outcomes: Vec<Pubkey>,
            // outcomes_name: Vec<&String>,
            outcomes_number: u8,
            expiration_time: u64,
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
            let mut final_outcomes: Vec<Outcome> = vec![];
            for n in 0..outcomes_number {
                final_outcomes.push(Outcome {
                    decimals: 8,
                    address: outcomes[usize::from(n)],
                    // ticker: outcomes_name[usize::from(n)].as_bytes().to_vec(),
                })
            }

            self.outcomes = final_outcomes;
            Ok(())
        }

        pub fn mint_complete_sets(&mut self, ctx: Context<Mint>, amount: u64) -> Result<()> {
            let outcomes: &[AccountInfo] = ctx.remaining_accounts;

            let deposited = ctx.accounts.collateral_account.amount;
            if deposited == 0 {
                return Err(ErrorCode::ZeroDeposit.into());
            }

            let seeds = &[self.signer.as_ref(), &[self.nonce]];
            let signer = &[&seeds[..]];

            for n in 0..outcomes.len() + 1 {
                let cpi_accounts = MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                };
                let cpi_program = ctx.accounts.token_program.to_account_info();
                let cpi_context = CpiContext::new(cpi_program, cpi_accounts).with_signer(signer);

                token::mint_to(cpi_context, amount)?;
            }

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
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    #[account(signer)]
    owner: AccountInfo<'info>,
    pub collateral_account: CpiAccount<'info, TokenAccount>,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Default, Clone)]
pub struct Outcome {
    pub address: Pubkey,
    pub decimals: u8,
    // pub ticker: Vec<u8>,
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
}
