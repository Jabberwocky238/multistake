use anchor_lang::prelude::*;
use crate::error::ErrorCode;
use super::item::PoolItem;
use static_assertions::const_assert_eq;
use std::mem::size_of;

/// 池中最多支持的质押类型数量
pub const MAX_TOKENS: usize = 512;

/// 单币质押池结构
///
/// 一个 Pool 对应一种主币，支持多种质押类型（items）
/// 使用 zero_copy 以避免栈溢出（大数组需要）
#[account(zero_copy)]
#[repr(C)]
#[derive(Debug)]
pub struct Pool {
    /// 实际使用的质押类型数量
    pub token_count: u16,
    /// 创建计数器 - 用于生成唯一的 LP mint seed，只增不减
    pub increment_count: u16,
    /// 填充字节（确保 8 字节对齐）
    pub padding: [u8; 4],
    /// Pool 管理员 - 用于所有操作的权限控制
    pub admin: Pubkey,
    /// Pool 的主币 Vault 账户 - 存储所有质押的主币
    pub pool_vault: Pubkey,
    /// Pool 的主币 Mint 地址 - 该 Pool 对应的币种
    pub pool_mint: Pubkey,
    /// 手续费分子
    pub fee_numerator: u64,
    /// 手续费分母
    pub fee_denominator: u64,
    /// 质押类型配置数组，最多支持 1024 种质押类型（固定大小）
    /// 每个 item 记录一种质押类型的 LP mint、已发行量和权重
    pub tokens: [PoolItem; MAX_TOKENS],
}

// 验证结构体大小和对齐（Solana 要求 8 字节对齐）
// 计算：2 + 2 + 4 + 32 + 32 + 32 + 8 + 8 + (48 * 512) = 24696 bytes
const_assert_eq!(
    size_of::<Pool>(),
    2 + 2 + 4 + 32 + 32 + 32 + 8 + 8 + (size_of::<PoolItem>() * MAX_TOKENS)
);
const_assert_eq!(size_of::<Pool>(), 24696);
const_assert_eq!(size_of::<Pool>() % 8, 0); // 必须是 8 的倍数

impl Pool {
    /// 验证管理员权限
    pub fn verify_admin(&self, admin: &Pubkey) -> Result<()> {
        require!(
            *admin == self.admin,
            crate::error::ErrorCode::InvalidAdmin
        );
        Ok(())
    }

    /// 获取实际使用的 token 数量
    pub fn get_token_count(&self) -> usize {
        self.token_count as usize
    }

    /// 根据 mint 地址查找 token 索引
    pub fn find_token_index(&self, mint: &Pubkey) -> Option<usize> {
        (0..self.get_token_count()).find(|&i| self.tokens[i].mint_account == *mint)
    }

    /// 根据索引获取 token item（可变引用）
    pub fn get_token_mut(&mut self, index: usize) -> Option<&mut PoolItem> {
        if index < self.get_token_count() {
            Some(&mut self.tokens[index])
        } else {
            None
        }
    }

    /// 根据 mint 地址获取 token item（不可变引用）
    pub fn get_token_by_mint(&self, mint: &Pubkey) -> Option<&PoolItem> {
        for i in 0..self.get_token_count() {
            if self.tokens[i].mint_account == *mint {
                return Some(&self.tokens[i]);
            }
        }
        None
    }

    /// 根据 mint 地址获取 token 索引
    pub fn get_token_index_by_mint(&self, mint: &Pubkey) -> Option<usize> {
        (0..self.get_token_count()).find(|&i| self.tokens[i].mint_account == *mint)
    }

    /// 根据索引获取 token item（不可变引用）
    pub fn get_token(&self, index: usize) -> Option<&PoolItem> {
        if index < self.get_token_count() {
            Some(&self.tokens[index])
        } else {
            None
        }
    }

    /// 添加新的质押类型（返回索引）
    /// lp_mint: 该质押类型的 LP 凭证 mint 地址
    /// weight: 该质押类型的初始权重
    pub fn add_token(&mut self, lp_mint: &Pubkey, weight: u64) -> Result<usize> {
        require!(
            self.get_token_count() < MAX_TOKENS,
            ErrorCode::MaxTokensReached
        );
        require!(weight > 0, ErrorCode::InvalidTokenCount);

        let index = self.get_token_count();
        let token = &mut self.tokens[index];
        token.set_mint_account(lp_mint);
        token.set_mint_amount(0); // 初始发行量为 0
        token.set_weight(weight);

        self.token_count += 1;
        Ok(index)
    }

    /// 计算账户所需的空间大小
    pub fn space() -> usize {
        8 + // discriminator
        2 + // token_count
        2 + // increment_count
        4 + // padding
        32 + // admin (Pubkey)
        32 + // pool_vault (Pubkey)
        32 + // pool_mint (Pubkey)
        8 + // fee_numerator
        8 + // fee_denominator
        (MAX_TOKENS * PoolItem::space()) // 固定大小数组
    }

    /// 获取手续费分子
    pub fn get_fee_numerator(&self) -> u64 {
        self.fee_numerator
    }

    /// 获取手续费分母
    pub fn get_fee_denominator(&self) -> u64 {
        self.fee_denominator
    }

    /// 设置费率
    pub fn set_fee(&mut self, fee_numerator: u64, fee_denominator: u64) {
        self.fee_numerator = fee_numerator;
        self.fee_denominator = fee_denominator;
    }

    /// 计算手续费
    /// amount: 输入金额
    /// 返回: (手续费金额, 扣除手续费后的金额)
    pub fn calculate_fee(&self, amount: u64) -> Result<(u64, u64)> {
        let amount_u128 = amount as u128;
        let fee_amount = amount_u128
            .checked_mul(self.fee_numerator as u128)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(self.fee_denominator as u128)
            .ok_or(ErrorCode::MathOverflow)?;
        let amount_after_fee = amount_u128
            .checked_sub(fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;
        
        Ok((fee_amount as u64, amount_after_fee as u64))
    }

    /// 计算所有质押类型的总加权质押量
    /// 返回: 所有类型的 (weight × mint_amount) 之和
    /// 公式: sum(weight_i × mint_amount_i)
    pub fn calculate_total_weighted_mint_amount(&self) -> Result<u128> {
        let token_count = self.get_token_count();
        let mut total_weighted: u128 = 0;

        for i in 0..token_count {
            if let Some(item) = self.get_token(i) {
                let mint_amount = item.get_mint_amount();
                if mint_amount > 0 {
                    let weight = item.get_weight() as u128;
                    let mint_amount_u128 = mint_amount as u128;

                    let weighted = weight
                        .checked_mul(mint_amount_u128)
                        .ok_or(ErrorCode::MathOverflow)?;

                    total_weighted = total_weighted
                        .checked_add(weighted)
                        .ok_or(ErrorCode::MathOverflow)?;
                }
            }
        }

        require!(total_weighted > 0, ErrorCode::InvalidTokenCount);
        Ok(total_weighted)
    }

    pub fn calculate_redeem_amount(
        &self,
        lp_amount: u64,
        item_index: usize,
        pool_vault_balance: u64,
    ) -> Result<u64> {
        require!(
            item_index < self.get_token_count(),
            ErrorCode::InvalidTokenIndex
        );

        let item = self.get_token(item_index)
            .ok_or(ErrorCode::InvalidTokenIndex)?;

        let weight = item.get_weight();
        let total_weighted = self.calculate_total_weighted_mint_amount()?;

        require!(weight > 0, ErrorCode::InvalidTokenCount);

        // 使用 u128 避免溢出
        let lp_amount_u128 = lp_amount as u128;
        let weight_u128 = weight as u128;
        let pool_vault_balance_u128 = pool_vault_balance as u128;

        let weighted_amount = lp_amount_u128
            .checked_mul(weight_u128)
            .ok_or(ErrorCode::MathOverflow)?;

        let redeem_amount = pool_vault_balance_u128
            .checked_mul(weighted_amount)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_div(total_weighted)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(redeem_amount as u64)
    }

    /// 获取 pool vault 的 Pubkey
    pub fn get_pool_vault(&self) -> &Pubkey {
        &self.pool_vault
    }

    /// 获取 pool mint 的 Pubkey
    pub fn get_pool_mint(&self) -> &Pubkey {
        &self.pool_mint
    }

    /// 设置 pool vault
    pub fn set_pool_vault(&mut self, vault: &Pubkey) {
        self.pool_vault = *vault;
    }

    /// 设置 pool mint
    pub fn set_pool_mint(&mut self, mint: &Pubkey) {
        self.pool_mint = *mint;
    }
}
