use borsh::{BorshDeserialize,BorshSerialize};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, MintTo, Transfer, Burn};

declare_id!("7hZHCToxNtH3vrx4Py7PZh7KUTaYJ6yDaUMq1AtL7EUi");

#[program]
pub mod solana_anchor {
    use super::*;

    pub fn initialize(
        ctx : Context<Initialize>,
        _bump : u8,
        _apy : u8
        ) -> ProgramResult {

        msg!("+Initialize");

        let pool = &mut ctx.accounts.pool;

        pool.owner = *ctx.accounts.owner.key;
        pool.rand = *ctx.accounts.rand.key;
        pool.bump = _bump;
        pool.token_mint = ctx.accounts.token_mint.key();
        pool.x_token_mint = ctx.accounts.x_token_mint.key();
        pool.apy = _apy;

        Ok(())
    }
    
    pub fn update(
        ctx : Context<Update>,
        _bump : u8,
        _apy : u8
        ) -> ProgramResult {

        msg!("+Update");

        let pool = &mut ctx.accounts.pool;

        pool.owner = *ctx.accounts.new_owner.key;
        pool.rand = *ctx.accounts.rand.key;
        pool.bump = _bump;
        pool.token_mint = ctx.accounts.token_mint.key();
        pool.x_token_mint = ctx.accounts.x_token_mint.key();
        pool.apy = _apy;

        Ok(())
    }

    pub fn init_user(
        ctx : Context<InitUser>,
        _bump : u8
        ) -> ProgramResult {
        msg!("+InitUser");

        let user_data = &mut ctx.accounts.user_data;

        user_data.owner = *ctx.accounts.owner.key;
        user_data.bump = _bump;
        user_data.stake_amount = 0;
        user_data.timestamp = 0;
        user_data.total_reward = 0;
        
        Ok(())
    }

    pub fn stake(
        ctx : Context<Stake>,
        amount : u64
        ) -> ProgramResult {

        msg!("+Stake");

        let pool = &ctx.accounts.pool;
        let user_data = &mut ctx.accounts.user_data;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];
        let pool_signer = &[&pool_signer_seeds[..]];

        // transfer reward from pool's token account to user's token account
        if user_data.timestamp > 0 && (clock.unix_timestamp as u64 - user_data.timestamp) / (24*60*60) != 0 {
            let reward_amount = user_data.stake_amount * (pool.apy as u64) / 100 / 365 * ((clock.unix_timestamp as u64 - user_data.timestamp) / (24*60*60));
            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: ctx.accounts.pool.to_account_info().clone(),
            };
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
            token::transfer(reward_cpi_ctx, reward_amount)?;

            user_data.total_reward += reward_amount;
        }

        // transfer tokens from user's token account to pool's token account
        let token_cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info().clone(),
            to: ctx.accounts.pool_token_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        let token_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let token_cpi_ctx = CpiContext::new(token_cpi_program, token_cpi_accounts);
        token::transfer(token_cpi_ctx, amount)?;

        user_data.stake_amount += amount;
        user_data.timestamp = clock.unix_timestamp as u64;
        
        // mint x_token to user's x_token account
        let mint_cpi_accounts = MintTo {
            mint: ctx.accounts.x_token_mint.to_account_info().clone(),
            to: ctx.accounts.user_x_token_account.to_account_info().clone(),
            authority: ctx.accounts.pool.to_account_info().clone(),
        };
        let mint_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let mint_cpi_ctx = CpiContext::new_with_signer(mint_cpi_program, mint_cpi_accounts, pool_signer);
        token::mint_to(mint_cpi_ctx, amount)?;

        Ok(())
    }

    pub fn un_stake(
        ctx : Context<UnStake>,
        amount : u64
        ) -> ProgramResult {

        msg!("+unstake");

        let pool = &ctx.accounts.pool;
        let user_data = &mut ctx.accounts.user_data;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];
        let pool_signer = &[&pool_signer_seeds[..]];

        // transfer reward from pool's token account to user's token account
        if user_data.timestamp > 0 && (((clock.unix_timestamp as u64 - user_data.timestamp) / (24*60*60)) as u8) != 0 {
            let reward_amount = user_data.stake_amount * (pool.apy as u64) / 100 / 365 * ((clock.unix_timestamp as u64 - user_data.timestamp) / (24*60*60));
            let reward_cpi_accounts = Transfer {
                from: ctx.accounts.pool_token_account.to_account_info().clone(),
                to: ctx.accounts.user_token_account.to_account_info().clone(),
                authority: ctx.accounts.pool.to_account_info().clone(),
            };
            let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
            let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
            token::transfer(reward_cpi_ctx, reward_amount)?;

            user_data.total_reward += reward_amount;
        }

        // burn x_token from user's x_token account
        let mut _amount = amount;
        if user_data.stake_amount < amount {
            _amount = user_data.stake_amount;
        }
        let burn_cpi_accounts = Burn {
            mint: ctx.accounts.x_token_mint.to_account_info().clone(),
            to: ctx.accounts.user_x_token_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        let burn_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let burn_cpi_ctx = CpiContext::new_with_signer(burn_cpi_program, burn_cpi_accounts, pool_signer);
        token::burn(burn_cpi_ctx, _amount)?;

        // transfer tokens from pool's token account to user's token account
        let token_cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info().clone(),
            to: ctx.accounts.user_token_account.to_account_info().clone(),
            authority: ctx.accounts.owner.to_account_info().clone(),
        };
        let token_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let token_cpi_ctx = CpiContext::new(token_cpi_program, token_cpi_accounts);
        token::transfer(token_cpi_ctx, _amount)?;

        user_data.stake_amount -= _amount;
        user_data.timestamp = clock.unix_timestamp as u64;

        Ok(())
    }

    pub fn claim_reward(
        ctx : Context<ClaimReward>
        ) -> ProgramResult {

        msg!("+claim reward");

        let pool = &ctx.accounts.pool;
        let user_data = &mut ctx.accounts.user_data;
        let clock = Clock::from_account_info(&ctx.accounts.clock)?;

        let pool_signer_seeds = &[
            pool.rand.as_ref(),
            &[pool.bump],
        ];
        let pool_signer = &[&pool_signer_seeds[..]];

        if user_data.timestamp == 0 {
            msg!("No stake amount");
            return Err(PoolError::NoStakeAmount.into());
        }

        if (((clock.unix_timestamp as u64 - user_data.timestamp) / (24*60*60)) as u8) != 0 {
            msg!("No reward amount");
            return Err(PoolError::NoRewardAmount.into());
        }

        let reward_amount = user_data.stake_amount * (pool.apy as u64) / 100 / 365 * ((clock.unix_timestamp as u64 - user_data.timestamp) / (24*60*60));
        let reward_cpi_accounts = Transfer {
            from: ctx.accounts.pool_token_account.to_account_info().clone(),
            to: ctx.accounts.user_token_account.to_account_info().clone(),
            authority: ctx.accounts.pool.to_account_info().clone(),
        };
        let reward_cpi_program = ctx.accounts.token_program.to_account_info().clone();
        let reward_cpi_ctx = CpiContext::new_with_signer(reward_cpi_program, reward_cpi_accounts, pool_signer);
        token::transfer(reward_cpi_ctx, reward_amount)?;

        user_data.total_reward += reward_amount;
        user_data.timestamp = clock.unix_timestamp as u64;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_bump : u8, _apy : u8)]
pub struct Initialize<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    #[account(init_if_needed, 
        seeds=[(*rand.key).as_ref()], 
        bump=_bump, payer=owner, 
        space=8+POOL_SIZE)]
    pool : ProgramAccount<'info, Pool>,

    rand : AccountInfo<'info>,

    #[account(owner=spl_token::id())]
    token_mint : Account<'info, Mint>,

    #[account(owner=spl_token::id())]
    x_token_mint : Account<'info, Mint>,

    system_program : Program<'info, System>
}

#[derive(Accounts)]
#[instruction(_bump : u8, _apy : u8)]
pub struct Update<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    new_owner : AccountInfo<'info>,

    #[account(mut, 
        has_one=owner,
        seeds=[(*rand.key).as_ref()], 
        bump=_bump)]
    pool : ProgramAccount<'info, Pool>,

    rand : AccountInfo<'info>,

    #[account(owner=spl_token::id())]
    token_mint : Account<'info, Mint>,

    #[account(owner=spl_token::id())]
    x_token_mint : Account<'info, Mint>,

    system_program : Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(_bump : u8)]
pub struct InitUser<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    #[account(init, 
        seeds=[(*owner.key).as_ref()], 
        bump=_bump, 
        payer=owner, 
        space=8+USER_DATA_SIZE)]
    user_data : ProgramAccount<'info,UserData>,

    system_program : Program<'info,System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    pool : ProgramAccount<'info, Pool>,

    #[account(mut, 
        has_one=owner,
        seeds=[(*owner.key).as_ref()],
    bump = user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(
        owner=spl_token::id(),
        constraint = token_mint.key() == pool.token_mint)]
    token_mint : Account<'info, Mint>,

    #[account(
        owner=spl_token::id(), 
        constraint = x_token_mint.key() == pool.x_token_mint)]
    x_token_mint : Account<'info, Mint>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint)]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint)]
    pool_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = user_x_token_account.owner == owner.key(),
        constraint = user_x_token_account.mint == pool.x_token_mint)]
    user_x_token_account:Account<'info, TokenAccount>,

    clock : AccountInfo<'info>,  

    token_program:Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UnStake<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    pool : ProgramAccount<'info, Pool>,

    #[account(mut, 
        has_one=owner,
        seeds=[(*owner.key).as_ref()], 
        bump=user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(
        owner=spl_token::id(),
        constraint = token_mint.key() == pool.token_mint)]
    token_mint : Account<'info, Mint>,

    #[account(
        owner=spl_token::id(), 
        constraint = x_token_mint.key() == pool.x_token_mint)]
    x_token_mint : Account<'info, Mint>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint)]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint)]
    pool_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = user_x_token_account.owner == owner.key(),
        constraint = user_x_token_account.mint == pool.x_token_mint)]
    user_x_token_account:Account<'info, TokenAccount>,

    clock : AccountInfo<'info>,  

    token_program:Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    owner : Signer<'info>,

    pool : ProgramAccount<'info, Pool>,

    #[account(mut, 
        has_one=owner,
        seeds=[(*owner.key).as_ref()],
        bump=user_data.bump)]
    user_data : ProgramAccount<'info, UserData>,

    #[account(
        owner=spl_token::id(),
        constraint = token_mint.key() == pool.token_mint)]
    token_mint : Account<'info, Mint>,

    #[account(mut,
        constraint = user_token_account.owner == owner.key(),
        constraint = user_token_account.mint == pool.token_mint)]
    user_token_account:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = pool_token_account.owner == pool.key(),
        constraint = pool_token_account.mint == pool.token_mint)]
    pool_token_account:Account<'info, TokenAccount>,

    clock : AccountInfo<'info>,  

    token_program:Program<'info, Token>,
}

pub const POOL_SIZE : usize = 32 + 32 + 1 + 32 + 32 + 1;
pub const USER_DATA_SIZE : usize = 32 + 1 + 8 + 8 + 8;

#[account]
pub struct Pool {
    pub owner : Pubkey,
    pub rand : Pubkey,
    pub bump : u8,
    pub token_mint : Pubkey,
    pub x_token_mint : Pubkey,
    pub apy : u8
}

#[account]
pub struct UserData {
    pub owner : Pubkey,
    pub bump : u8,
    pub stake_amount : u64,
    pub timestamp : u64,
    pub total_reward : u64
}

#[error]
pub enum PoolError {
    #[msg("Token mint to failed")]
    TokenMintToFailed,

    #[msg("Token set authority failed")]
    TokenSetAuthorityFailed,

    #[msg("Token transfer failed")]
    TokenTransferFailed,

    #[msg("Invalid token amount")]
    InvalidTokenAmount,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid metadata")]
    InvalidMetadata,

    #[msg("Invalid stakedata account")]
    InvalidStakeData,

    #[msg("Invalid time")]
    InvalidTime,

    #[msg("Invalid owner")]
    InvalidOwner,

    #[msg("No stake amount")]
    NoStakeAmount,

    #[msg("No reward amount")]
    NoRewardAmount
}