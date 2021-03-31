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
        pub admin: Pubkey,
        pub mint_authority: Pubkey,
        pub collateral_token: Pubkey,
        pub collateral_account: Pubkey,
        pub outcomes: Vec<Outcome>,
    }

    impl InternalState {
        pub const ASSETS_SIZE: usize = 10;
        pub fn new(_ctx: Context<New>) -> Result<Self> {
            let mut outcomes: Vec<Outcome> = vec![];
            outcomes.resize(
                Self::ASSETS_SIZE,
                Outcome {
                    ticker: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                    ..Default::default()
                },
            );
            Ok(Self {
                nonce: 0,
                signer: Pubkey::default(),
                admin: Pubkey::default(),
                mint_authority: Pubkey::default(),
                collateral_token: Pubkey::default(),
                collateral_account: Pubkey::default(),
                outcomes,
            })
        }
        pub fn initialize(
            &mut self,
            _ctx: Context<Initialize>,
            nonce: u8,
            signer: Pubkey,
            admin: Pubkey,
            collateral_token: Pubkey,
            collateral_account: Pubkey,
            usd_token: Pubkey,
            mint_authority: Pubkey,
            outcomes: Vec<Pubkey>,
            outcomes_name: Vec<Vec<u8>>,
            outcomes_number: u8,
        ) -> Result<()> {
            self.signer = signer;
            self.nonce = nonce;
            self.admin = admin;
            self.collateral_token = collateral_token;
            self.collateral_account = collateral_account;
            self.mint_authority = mint_authority;
            let mut outcomes: Vec<Outcome> = vec![];
            for n in 0..outcomes_number {
                outcomes.push(Outcome {
                    decimals: 8,
                    address: outcomes[n],
                    ticker: outcomes_name[n].as_bytes().to_vec(),
                })
            }

            self.outcomes = outcomes;
            Ok(())
        }

        pub fn mint(&mut self, ctx: Context<Mint>, amount: u64) -> Result<()> {
            /*
            let deposited = ctx.accounts.collateral_account.amount;
            if deposited == 0 {
                return Err(ErrorCode::ZeroDeposit.into());
            }
            */

            let mint_token_adddress = ctx.accounts.mint.to_account_info().clone().key;

            if !mint_token_adddress.eq(&self.outcomes[0].address) {
                return Err(ErrorCode::NotSyntheticUsd.into());
            }

            let seeds = &[self.signer.as_ref(), &[self.nonce]];
            let signer = &[&seeds[..]];

            let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer);
            token::mint_to(cpi_ctx, amount)?;

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
}
impl<'a, 'b, 'c, 'info> From<&Mint<'info>> for CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
    fn from(accounts: &Mint<'info>) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: accounts.mint.to_account_info(),
            to: accounts.to.to_account_info(),
            authority: accounts.authority.to_account_info(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Default, Clone)]
pub struct Outcome {
    pub address: Pubkey,
    pub decimals: u8,
    pub ticker: Vec<u8>,
}

#[error]
pub enum ErrorCode {
    #[msg("Mint limit crossed")]
    MintLimit,
    #[msg("Wrong token not sythetic usd")]
    NotSyntheticUsd,
    #[msg("Deposited zero")]
    ZeroDeposit,
}
