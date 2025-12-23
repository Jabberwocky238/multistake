use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};
use crate::state::AnySwapPool;
use crate::error::ErrorCode;

/// 销毁 LP 凭证，赎回主币
#[derive(Accounts)]
#[instruction(item_index: u16)]
pub struct Unstake<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, AnySwapPool>,

    /// Pool authority PDA
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

    /// 用户的 LP 凭证账户（销毁来源）
    #[account(mut)]
    pub user_lp_token: Box<Account<'info, TokenAccount>>,

    /// 用户的主币账户（赎回目标）
    #[account(mut)]
    pub user_main_token: Box<Account<'info, TokenAccount>>,

    /// 用户签名
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// 销毁 LP 凭证，赎回主币
/// item_index: 质押类型索引
/// lp_amount: 要销毁的 LP 凭证数量
///
/// 逻辑：
/// 1. 销毁用户的 LP 凭证
/// 2. 根据 weight 计算能赎回的主币数量
/// 3. 从 pool_vault 转移主币给用户
/// 4. 更新 item 的 mint_amount
pub fn unstake(
    ctx: Context<Unstake>,
    item_index: u16,
    lp_amount: u64,
) -> Result<()> {
    require!(lp_amount > 0, ErrorCode::InvalidTokenCount);

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

    // 计算能赎回的主币数量（基于 weight）
    let pool_vault_balance = ctx.accounts.pool_vault.amount;
    let redeem_amount = pool.calculate_redeem_amount(
        lp_amount,
        item_index as usize,
        pool_vault_balance,
    )?;

    require!(
        pool_vault_balance >= redeem_amount,
        ErrorCode::InsufficientLiquidity
    );

    // 1. 销毁用户的 LP 凭证
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        lp_amount,
    )?;

    // 2. 从 pool_vault 转移主币给用户
    let pool_key = ctx.accounts.pool.key();
    let bump = ctx.bumps.pool_authority;
    let seeds = &[
        b"anyswap_authority",
        pool_key.as_ref(),
        &[bump],
    ];
    let signer = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_vault.to_account_info(),
                to: ctx.accounts.user_main_token.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer,
        ),
        redeem_amount,
    )?;

    // 3. 更新 item 的 mint_amount
    let item_mut = pool.get_token_mut(item_index as usize)
        .ok_or(ErrorCode::InvalidTokenIndex)?;
    item_mut.sub_mint_amount(lp_amount)?;

    msg!("Unstaked: user: {}, item_index: {}, lp_burned: {}, main_token_redeemed: {}",
         ctx.accounts.user.key(),
         item_index,
         lp_amount,
         redeem_amount);

    Ok(())
}

