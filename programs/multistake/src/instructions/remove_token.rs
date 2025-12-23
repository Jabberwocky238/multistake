use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::Pool;
use crate::error::ErrorCode;

/// 从 pool 中移除质押类型
#[derive(Accounts)]
pub struct RemoveTokenFromPool<'info> {
    #[account(mut)]
    pub pool: AccountLoader<'info, Pool>,

    /// 要移除的 LP mint 账户
    pub lp_mint: Account<'info, Mint>,

    /// Pool 管理员 - 必须签名
    pub admin: Signer<'info>,
}

/// 从 pool 中移除质押类型
/// 注意：移除前需要确保该类型的 LP 已全部销毁（mint_amount = 0）
pub fn remove_token_from_pool(ctx: Context<RemoveTokenFromPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool.load_mut()?;

    // 验证管理员权限
    pool.verify_admin(&ctx.accounts.admin.key())?;

    let lp_mint_key = ctx.accounts.lp_mint.key();
    let token_index = pool.find_token_index(&lp_mint_key)
        .ok_or(ErrorCode::InvalidTokenMint)?;

    // 检查是否是最后一个 token
    let token_count = pool.get_token_count();
    require!(token_count > 0, ErrorCode::InvalidTokenCount);

    // 验证 LP mint 地址是否匹配
    let token_item = pool.get_token(token_index).ok_or(ErrorCode::InvalidTokenIndex)?;
    require!(
        ctx.accounts.lp_mint.key() == *token_item.mint_pubkey(),
        ErrorCode::InvalidTokenMint
    );

    // 检查该类型的 LP 是否已全部销毁
    require!(
        token_item.get_mint_amount() == 0,
        ErrorCode::InsufficientTokenAmount
    );

    // 如果是最后一个 token，直接减少计数
    if token_index == token_count - 1 {
        pool.token_count -= 1;
    } else {
        // 如果不是最后一个，将最后一个 token 移动到当前位置
        let last_index = token_count - 1;

        // 先获取最后一个 token 的数据（通过索引访问，避免借用冲突）
        let last_token_data = pool.tokens[last_index];

        // 复制最后一个 token 到当前位置
        pool.tokens[token_index] = last_token_data;

        // 减少计数
        pool.token_count -= 1;
    }

    msg!("Staking type removed from pool: lp_mint: {}", lp_mint_key);
    Ok(())
}

