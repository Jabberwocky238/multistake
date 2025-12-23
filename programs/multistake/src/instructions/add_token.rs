use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use crate::state::Pool;
use crate::error::ErrorCode;

/// 添加质押类型到 pool
/// 自动创建新的 LP mint，权限归属于 pool authority
#[derive(Accounts)]
pub struct AddTokenToPool<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// Pool authority PDA - LP mint 的 authority
    /// CHECK: PDA derived from pool key
    #[account(
        seeds = [b"anyswap_authority", pool.key().as_ref()],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    /// LP mint - 自动创建，权限归属于 pool_authority
    /// 使用 increment_count 作为 seed 确保唯一性（只增不减）
    #[account(
        init,
        payer = payer,
        mint::decimals = 9,
        mint::authority = pool_authority,
    )]
    pub lp_mint: Account<'info, Mint>,

    /// Pool 管理员 - 必须签名
    pub admin: Signer<'info>,

    /// 支付创建 LP mint 的费用
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// 添加质押类型到 pool
///
/// 自动创建 LP mint（由 Anchor 处理）
/// weight 默认为 10^8 (100,000,000)
/// mint_amount 初始为 0
pub fn add_token_to_pool(ctx: Context<AddTokenToPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool.load_mut()?;

    // 验证管理员权限
    pool.verify_admin(&ctx.accounts.admin.key())?;

    // 默认权重：10^8 = 100,000,000
    const DEFAULT_WEIGHT: u64 = 100_000_000;

    // 添加质押类型（LP mint 和默认 weight）
    let lp_mint_key = ctx.accounts.lp_mint.key();
    let index = pool.add_token(&lp_mint_key, DEFAULT_WEIGHT)?;

    // increment_count 递增（只增不减）
    pool.increment_count = pool.increment_count
        .checked_add(1)
        .ok_or(ErrorCode::MathOverflow)?;

    msg!("Staking type added: index: {}, lp_mint: {}, weight: {}, mint_amount: 0",
         index, lp_mint_key, DEFAULT_WEIGHT);

    Ok(())
}

