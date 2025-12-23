use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

/// 质押类型配置项
/// 每个 item 记录一种质押类型的 LP mint、已发行量和权重
/// 用于单币质押系统，不同质押类型有不同的收益权重
#[zero_copy]
#[repr(C)]
#[derive(Debug)]
pub struct AnySwapItem {
    /// LP 凭证 Mint account 地址 - 用户质押后获得的凭证 token (32 bytes)
    pub mint_account: Pubkey, // 32 bytes
    /// 已铸造的 LP 凭证数量 - 该质押类型的总发行量 (8 bytes)
    pub mint_amount: u64, // 8 bytes
    /// 权重 (weight) - 动态权重，由 admin 通过 oracle 修改 (8 bytes)
    /// 影响 LP 凭证兑换主币的比率，weight 越高收益越好
    pub weight: u64, // 8 bytes
}

// 验证结构体大小和对齐（Solana 要求 8 字节对齐）
const_assert_eq!(size_of::<AnySwapItem>(), 32 + 8 + 8); // 48 bytes
const_assert_eq!(size_of::<AnySwapItem>() % 8, 0); // 必须是 8 的倍数

impl AnySwapItem {
    /// 检查 item 是否为空（未使用）
    pub fn is_empty(&self) -> bool {
        self.mint_account == Pubkey::default()
    }

    /// 获取 LP mint account 的 Pubkey
    pub fn mint_pubkey(&self) -> &Pubkey {
        &self.mint_account
    }

    /// 获取已铸造的 LP 凭证数量
    pub fn get_mint_amount(&self) -> u64 {
        self.mint_amount
    }

    /// 获取 weight 值
    pub fn get_weight(&self) -> u64 {
        self.weight
    }

    /// 设置 weight 值（由 admin 通过 oracle 动态修改）
    pub fn set_weight(&mut self, weight: u64) {
        self.weight = weight;
    }

    /// 设置 LP mint account
    pub fn set_mint_account(&mut self, pubkey: &Pubkey) {
        self.mint_account = *pubkey;
    }

    /// 设置已铸造的 LP 凭证数量
    pub fn set_mint_amount(&mut self, amount: u64) {
        self.mint_amount = amount;
    }

    /// 增加已铸造的 LP 凭证数量
    pub fn add_mint_amount(&mut self, amount: u64) -> Result<()> {
        self.mint_amount = self.mint_amount
            .checked_add(amount)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;
        Ok(())
    }

    /// 减少已铸造的 LP 凭证数量
    pub fn sub_mint_amount(&mut self, amount: u64) -> Result<()> {
        self.mint_amount = self.mint_amount
            .checked_sub(amount)
            .ok_or(crate::error::ErrorCode::MathOverflow)?;
        Ok(())
    }

    /// 计算单个 item 所需的空间大小
    pub fn space() -> usize {
        32 + // mint_account (Pubkey)
        8 + // mint_amount
        8 // weight
    }
}

