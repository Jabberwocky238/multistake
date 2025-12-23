use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Transfer};
use crate::state::Pool;
use crate::error::ErrorCode;

/// 质押主币，铸造 LP 凭证
#[derive(Accounts)]
#[instruction(item_index: u16)]
pub struct Stake<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// Pool authority PDA - LP mint 的 authority
    /// CHECK: PDA derived from pool key
    #[account(
        seeds = [b"anyswap_authority", pool.key().as_ref()],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    /// Pool 的主币 Vault
    #[account(
        mut,
        seeds = [b"pool_vault", pool.key().as_ref()],
        bump,
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    /// LP mint - 对应的质押类型
    /// 通过 pool.get_token() 验证地址是否匹配
    #[account(mut)]
    pub lp_mint: Box<Account<'info, Mint>>,

    /// 用户的主币账户（质押来源）
    #[account(mut)]
    pub user_main_token: Box<Account<'info, TokenAccount>>,

    /// 用户的 LP 凭证账户（铸造目标）
    #[account(mut)]
    pub user_lp_token: Box<Account<'info, TokenAccount>>,

    /// 用户签名
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// 质押主币，铸造 LP 凭证
/// item_index: 质押类型索引
/// stake_amount: 质押的主币数量
///
/// 逻辑：
/// 1. 用户转移主币到 pool_vault
/// 2. 铸造等量的 LP 凭证给用户（1:1）
/// 3. 更新 item 的 mint_amount
pub fn stake(
    ctx: Context<Stake>,
    item_index: u16,
    stake_amount: u64,
) -> Result<()> {
    require!(stake_amount > 0, ErrorCode::InvalidTokenCount);

    let pool = &mut ctx.accounts.pool.load_mut()?;

    // 验证 item_index 有效
    require!(
        (item_index as usize) < pool.get_token_count(),
        ErrorCode::InvalidTokenIndex
    );

    // 验证 LP mint 地址匹配
    let item = pool.get_token(item_index as usize)
        .ok_or(ErrorCode::InvalidTokenIndex)?;
    require!(
        ctx.accounts.lp_mint.key() == *item.mint_pubkey(),
        ErrorCode::InvalidTokenMint
    );

    // 计算手续费
    let (fee_amount, amount_after_fee) = pool.calculate_fee(stake_amount)?;

    // 1. 用户转移全额主币到 pool_vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_main_token.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        stake_amount,
    )?;

    // 2. 铸造扣除手续费后的 LP 凭证给用户
    let pool_key = ctx.accounts.pool.key();
    let bump = ctx.bumps.pool_authority;
    let seeds = &[
        b"anyswap_authority",
        pool_key.as_ref(),
        &[bump],
    ];
    let signer = &[&seeds[..]];

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer,
        ),
        amount_after_fee,
    )?;

    // 3. 更新 item 的 mint_amount（只记录扣除手续费后的数量）
    let item_mut = pool.get_token_mut(item_index as usize)
        .ok_or(ErrorCode::InvalidTokenIndex)?;
    item_mut.add_mint_amount(amount_after_fee)?;

    msg!("Staked: user: {}, item_index: {}, amount: {}, fee: {}, lp_minted: {}",
         ctx.accounts.user.key(),
         item_index,
         stake_amount,
         fee_amount,
         amount_after_fee);

    Ok(())
}

