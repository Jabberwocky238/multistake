use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::AnySwapPool;
use crate::error::ErrorCode;

/// 创建单币质押 Pool
/// 每个 Pool 对应一种主币，支持多种质押类型（最多 512 种）
#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(zero)]
    pub pool: AccountLoader<'info, AnySwapPool>,

    /// Pool authority PDA - 用于管理 pool vault
    /// CHECK: 用于管理 pool vault
    #[account(
        seeds = [b"anyswap_authority", pool.key().as_ref()],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,

    /// 主币的 Mint 账户 - Pool 对应的币种
    pub main_token_mint: Account<'info, Mint>,

    /// Pool 的主币 Vault - 存储所有质押的主币
    #[account(
        init,
        payer = payer,
        seeds = [b"pool_vault", pool.key().as_ref()],
        bump,
        token::mint = main_token_mint,
        token::authority = pool_authority
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    /// Pool 管理员 - 用于所有操作的权限控制
    pub admin: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// 创建 Pool
/// fee_numerator: 手续费分子
/// fee_denominator: 手续费分母
/// 例如：fee_numerator=3, fee_denominator=1000 表示 0.3% 手续费
pub fn create_pool(
    ctx: Context<CreatePool>,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<()> {
    require!(fee_denominator > 0, ErrorCode::MathOverflow);
    require!(fee_numerator <= fee_denominator, ErrorCode::MathOverflow);
    
    let pool = &mut ctx.accounts.pool.load_init()?;
    pool.token_count = 0;
    pool.increment_count = 0;
    pool.padding = [0u8; 4];
    pool.admin = ctx.accounts.admin.key();
    pool.pool_vault = ctx.accounts.pool_vault.key();
    pool.pool_mint = ctx.accounts.main_token_mint.key();
    pool.fee_numerator = fee_numerator;
    pool.fee_denominator = fee_denominator;

    // 初始化所有质押类型 items 为零值（zero_copy 会自动处理）

    msg!("Staking Pool created: pool: {}, main_token_mint: {}, pool_vault: {}, admin: {}, fee: {}/{}",
         ctx.accounts.pool.key(),
         ctx.accounts.main_token_mint.key(),
         ctx.accounts.pool_vault.key(),
         ctx.accounts.admin.key(),
         fee_numerator,
         fee_denominator);
    Ok(())
}

