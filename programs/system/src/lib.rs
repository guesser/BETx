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
        // pub total_collateral_shares: u64,
        pub collateral_token: Pubkey,
        pub collateral_account: Pubkey,
        pub assets: Vec<Asset>,
    }

    impl InternalState {
        pub const ASSETS_SIZE: usize = 10;
        pub fn new(_ctx: Context<New>) -> Result<Self> {
            let mut assets: Vec<Asset> = vec![];
            assets.resize(
                Self::ASSETS_SIZE,
                Asset {
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
                assets,
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
        ) -> Result<()> {
            self.signer = signer;
            self.nonce = nonce;
            self.admin = admin;
            self.collateral_token = collateral_token;
            self.collateral_account = collateral_account;
            self.mint_authority = mint_authority;
            //clean asset array + add synthetic Usd
            let usd_asset = Asset {
                decimals: 8,
                asset_address: usd_token,
                last_update: std::u64::MAX,
                price: 1 * 10u64.pow(4),
                supply: 0,
                ticker: "xUSD".as_bytes().to_vec(),
            };
            let collateral_asset = Asset {
                decimals: 8,
                asset_address: collateral_token,
                last_update: 0,
                price: 0,
                supply: 0,
                ticker: "SNY".as_bytes().to_vec(),
            };
            self.assets = vec![usd_asset, collateral_asset];
            Ok(())
        }

        // This only support sythetic USD
        pub fn mint(&mut self, ctx: Context<Mint>, amount: u64) -> Result<()> {
            /*
            let deposited = ctx.accounts.collateral_account.amount;
            if deposited == 0 {
                return Err(ErrorCode::ZeroDeposit.into());
            }
            */

            let mint_token_adddress = ctx.accounts.mint.to_account_info().clone().key;

            if !mint_token_adddress.eq(&self.assets[0].asset_address) {
                return Err(ErrorCode::NotSyntheticUsd.into());
            }

            let seeds = &[self.signer.as_ref(), &[self.nonce]];
            let signer = &[&seeds[..]];

            let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer);
            token::mint_to(cpi_ctx, amount)?;

            Ok(())
        }
    }

    // Change to create market
    pub fn create_user_account(ctx: Context<CreateUserAccount>, owner: Pubkey) -> ProgramResult {
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = owner;
        user_account.shares = 0;
        user_account.collateral = 0;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct New {}
#[derive(Accounts)]
pub struct Initialize {}
#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(init)]
    pub user_account: ProgramAccount<'info, UserAccount>,
    pub rent: Sysvar<'info, Rent>,
}
#[derive(Accounts)]
pub struct Mint<'info> {
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    pub user_account: ProgramAccount<'info, UserAccount>,
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
#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub shares: u64,
    pub collateral: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Default, Clone)]
pub struct Asset {
    pub asset_address: Pubkey,
    pub price: u64,
    pub last_update: u64,
    pub supply: u64,
    pub decimals: u8,
    pub ticker: Vec<u8>,
}
#[derive(Accounts)]
pub struct Deposit<'info> {
    // #[account(signer)]
    // pub test: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: ProgramAccount<'info, UserAccount>,
    pub collateral_account: CpiAccount<'info, TokenAccount>,
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
