use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod error;

use instructions::*;
declare_id!("2mgSDKAjDo8fQN6oms6YzczHhyeYEJunTzxjQgegYADf");

#[program]
pub mod multistake {
    use super::*;

    /// 创建 Pool（PDA）
    pub fn create_pool(
        ctx: Context<CreatePool>,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<()> {
        instructions::create_pool(ctx, fee_numerator, fee_denominator)
    }

    /// 添加质押类型到 Pool
    /// 自动创建 LP mint，使用 increment_count 作为 seed
    /// 权重默认 10^8
    pub fn add_token_to_pool(
        ctx: Context<AddTokenToPool>,
    ) -> Result<()> {
        instructions::add_token_to_pool(ctx)
    }

    /// 从 MultiStake Pool 移除 token
    pub fn remove_token_from_pool(
        ctx: Context<RemoveTokenFromPool>,
    ) -> Result<()> {
        instructions::remove_token_from_pool(ctx)
    }

    /// 修改 token 的 weight
    pub fn modify_token_weight(
        ctx: Context<ModifyTokenWeight>,
        new_weights: Vec<u64>,
    ) -> Result<()> {
        instructions::modify_token_weight(ctx, new_weights)
    }

    /// 质押主币，铸造 LP 凭证
    pub fn stake(
        ctx: Context<Stake>,
        item_index: u16,
        stake_amount: u64,
    ) -> Result<()> {
        instructions::stake(ctx, item_index, stake_amount)
    }

    /// 销毁 LP 凭证，赎回主币
    pub fn unstake(
        ctx: Context<Unstake>,
        item_index: u16,
        lp_amount: u64,
    ) -> Result<()> {
        instructions::unstake(ctx, item_index, lp_amount)
    }
}
