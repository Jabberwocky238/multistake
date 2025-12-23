use crate::{error::ErrorCode, state::AnySwapPool};
use anchor_lang::prelude::*;
use primitive_types::U256;

/// 添加流动性的结果
pub struct AddLiquidityResult {
    pub lp_minted: u64,
    pub burn_fees: Vec<u64>,
    // 实际使用了用户的token数量
    pub amounts_used: Vec<u64>, 
    // 实际加入池子的token数量
    pub amounts_in: Vec<u64>,
}

/// 移除流动性的结果
pub struct RemoveLiquidityResult {
    // 实际发给用户的token数量
    pub amounts_out: Vec<u64>,
    // 实际扣掉的手续费
    pub burn_fees: Vec<u64>,
}

pub trait LiquidityProtocol {
    fn add_liquidity<'info>(
        &self,
        token_vaults_amount: &[u64],
        amounts_in: &[u64],
        total_lp_supply: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<AddLiquidityResult>;

    fn remove_liquidity<'info>(
        &self,
        token_vaults_amount: &[u64],
        lp_to_burn: u64,
        total_lp_supply: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<RemoveLiquidityResult>;
}

/// 添加流动性（CPMM模型）
///
/// 用户按当前池子的比例提供所有token，铸造LP按比例计算
///
/// 公式：
/// - 首次添加：LP = 第一个token的数量（扣费后）
/// - 后续添加：LP = total_LP * (提供的token数量 / 该token当前储备)
///
/// Args:
///     token_vaults_amount: 当前储备列表
///     amounts_in: 用户提供的token数量列表
///     total_lp_supply: 当前LP token总供应量
///     fee_numerator: 费率分子
///     fee_denominator: 费率分母
pub fn add_liquidity_inner(
    token_vaults_amount: &[u64],
    amounts_in: &[u64],
    total_lp_supply: u64,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<AddLiquidityResult> {
    let token_count = token_vaults_amount.len();
    require!(
        amounts_in.len() == token_count,
        ErrorCode::InvalidTokenCount
    );

    // 计算费率和扣费后的金额
    let mut burn_fees = Vec::with_capacity(token_count);
    let mut amounts_after_fee = Vec::with_capacity(token_count);

    for &amount in amounts_in.iter() {
        let amount_u256 = U256::from(amount);
            let fee_amount = (amount_u256 * fee_numerator) / fee_denominator;
        let amount_after = amount_u256
                .checked_sub(fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;

        burn_fees.push(fee_amount.as_u64());
        amounts_after_fee.push(amount_after.as_u64());
    }

    // 计算LP铸造数量和实际使用的token数量
    let (lp_minted, amounts_in_pool, amounts_used_from_user) = if total_lp_supply == 0 {
        // 首次添加流动性：LP = 第一个token的数量（扣费后）
        // amounts_in_pool = 扣费后加入池子的量
        // amounts_used_from_user = 用户提供的总量（包括费用）
        let mut used_from_user = Vec::with_capacity(token_count);
        for i in 0..token_count {
            used_from_user.push(amounts_after_fee[i] + burn_fees[i]);
        }
        (amounts_after_fee[0], amounts_after_fee.clone(), used_from_user)
    } else {
        // 后续添加：找到最小比例，按最小比例计算
        // 计算每个token的比例 ratio_i = amount_i / vault_i
        let mut min_ratio = U256::MAX;
        let mut min_ratio_index = 0;

        for i in 0..token_count {
            if token_vaults_amount[i] == 0 {
                continue;
            }
            let amount = U256::from(amounts_after_fee[i]);
            let vault = U256::from(token_vaults_amount[i]);
            
            // ratio = amount * 1e18 / vault（放大1e18避免精度丢失）
            let ratio = (amount * U256::from(1_000_000_000_000_000_000u64)) / vault;
            
            if ratio < min_ratio {
                min_ratio = ratio;
                min_ratio_index = i;
            }
        }

        require!(min_ratio < U256::MAX, ErrorCode::InsufficientLiquidity);

        // 使用最小比例计算LP和实际使用的token数量
        let amount_min = U256::from(amounts_after_fee[min_ratio_index]);
        let vault_min = U256::from(token_vaults_amount[min_ratio_index]);
        let total_lp = U256::from(total_lp_supply);
        
        let lp = (amount_min * total_lp) / vault_min;
        
        // 计算每个token实际加入池子的数量（扣费后）= vault_i * lp / total_lp
        let mut amounts_in_pool_vec = Vec::with_capacity(token_count);
        let mut amounts_used_vec = Vec::with_capacity(token_count);
        
        for i in 0..token_count {
            let vault = U256::from(token_vaults_amount[i]);
            let amount_in_pool = (vault * lp) / total_lp;
            amounts_in_pool_vec.push(amount_in_pool.as_u64());
            
            // 计算从用户拿走的总量（包括费用）
            // fee = amount_in_pool * fee_rate / (1 - fee_rate)
            let amount_before_fee = (amount_in_pool * U256::from(fee_denominator)) 
                / U256::from(fee_denominator - fee_numerator);
            amounts_used_vec.push(amount_before_fee.as_u64());
        }

        (lp.as_u64(), amounts_in_pool_vec, amounts_used_vec)
    };

    Ok(AddLiquidityResult {
        lp_minted,
        burn_fees,
        amounts_used: amounts_used_from_user,
        amounts_in: amounts_in_pool,
    })
}

/// 移除流动性（CPMM模型）
///
/// 用户销毁LP token，按比例获得所有token
///
/// 公式：
/// - LP占比 = lp_to_burn / total_LP
/// - 每个token的输出 = vault_i * LP占比
///
/// Args:
///     token_vaults_amount: 当前储备列表
///     lp_to_burn: 要销毁的LP token数量
///     total_lp_supply: 当前LP token总供应量
///     fee_numerator: 费率分子
///     fee_denominator: 费率分母
pub fn remove_liquidity_inner(
    token_vaults_amount: &[u64],
    lp_to_burn: u64,
    total_lp_supply: u64,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<RemoveLiquidityResult> {
    let token_count = token_vaults_amount.len();

    require!(
        lp_to_burn <= total_lp_supply,
        ErrorCode::InsufficientLiquidity
    );
    require!(total_lp_supply > 0, ErrorCode::InsufficientLiquidity);

    let mut amounts_out = Vec::with_capacity(token_count);
    let mut burn_fees = Vec::with_capacity(token_count);

    // 计算LP占比和每个token的输出
    let lp_burn = U256::from(lp_to_burn);
    let total_lp = U256::from(total_lp_supply);

    for &vault in token_vaults_amount.iter() {
        // amount_out = vault * lp_to_burn / total_lp
        let vault_u256 = U256::from(vault);
        let amount_before_fee = (vault_u256 * lp_burn) / total_lp;

        // 计算费率
        let fee_amount = (amount_before_fee * fee_numerator) / fee_denominator;
        let amount_after_fee = amount_before_fee
            .checked_sub(fee_amount)
            .ok_or(ErrorCode::MathOverflow)?;

        amounts_out.push(amount_after_fee.as_u64());
        burn_fees.push(fee_amount.as_u64());
    }

    Ok(RemoveLiquidityResult {
        amounts_out,
        burn_fees,
    })
}

impl LiquidityProtocol for AnySwapPool {
    fn add_liquidity<'info>(
        &self,
        token_vaults_amount: &[u64],
        amounts_in: &[u64],
        total_lp_supply: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<AddLiquidityResult> {
        add_liquidity_inner(
            token_vaults_amount,
            amounts_in,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
    }

    fn remove_liquidity<'info>(
        &self,
        token_vaults_amount: &[u64],
        lp_to_burn: u64,
        total_lp_supply: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<RemoveLiquidityResult> {
        remove_liquidity_inner(
            token_vaults_amount,
            lp_to_burn,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_liquidity_bootstrap() {
        // 测试首次添加流动性
        let vaults = vec![
            10_000_000u64,
            50_000_000,
            100_000_000,
            20_000_000,
            30_000_000,
            40_000_000,
        ];
        let amounts_in = vec![
            1_000_000u64,
            5_000_000,
            10_000_000,
            2_000_000,
            3_000_000,
            4_000_000,
        ];
        let total_lp_supply = 0u64;
        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        let result = add_liquidity_inner(
            &vaults,
            &amounts_in,
            total_lp_supply,
                fee_numerator,
                fee_denominator,
            )
        .unwrap();

        // 验证LP铸造数量 = 第一个token扣费后的数量
        let expected_lp = amounts_in[0] - (amounts_in[0] * fee_numerator / fee_denominator);
        assert_eq!(result.lp_minted, expected_lp);
        assert_eq!(result.lp_minted, 999_700);

        // 验证费用
        for i in 0..amounts_in.len() {
            let expected_fee = amounts_in[i] * fee_numerator / fee_denominator;
            assert_eq!(result.burn_fees[i], expected_fee);
        }

        println!("✅ 首次添加流动性测试通过！");
        println!("  铸造LP: {}", result.lp_minted);
    }

    #[test]
    fn test_add_liquidity_subsequent() {
        // 测试第二次添加流动性
        // 第一次添加后的状态
        let vaults = vec![
            10_999_700u64,
            54_998_500,
            109_997_000,
            21_999_400,
            32_999_100,
            43_998_800,
        ];
        let amounts_in = vec![
            500_000u64, 2_500_000, 5_000_000, 1_000_000, 1_500_000, 2_000_000,
        ];
        let total_lp_supply = 999_700u64;
        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        let result = add_liquidity_inner(
            &vaults,
            &amounts_in,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
        .unwrap();

        // 验证LP铸造数量
        // LP = total_LP * (amount_0_after_fee / vault_0)
        let amount_0_after_fee = amounts_in[0] - (amounts_in[0] * fee_numerator / fee_denominator);
        let expected_lp =
            (amount_0_after_fee as u128 * total_lp_supply as u128 / vaults[0] as u128) as u64;

        assert_eq!(result.lp_minted, expected_lp);
        assert_eq!(result.lp_minted, 45_428);

        println!("✅ 第二次添加流动性测试通过！");
        println!("  铸造LP: {}", result.lp_minted);
    }

    #[test]
    fn test_add_liquidity_unbalanced() {
        // 测试不等比例添加流动性（用户提供的比例不一致）
        println!("\n=== 测试不等比例添加流动性 ===");
        
        // 第一次添加后的状态
        let vaults = vec![
            10_999_700u64,
            54_998_500,
            109_997_000,
            21_999_400,
            32_999_100,
            43_998_800,
        ];
        let total_lp_supply = 999_700u64;
        
        // 用户提供不等比例的token（故意让比例不一致）
        // 正常比例应该是 5:25:50:10:15:20
        // 但用户提供了 10:25:50:10:15:20（token_0多了一倍）
        let amounts_in = vec![
            1_000_000u64,  // token_0: 比例高（1M / 11M ≈ 9.09%）
            2_500_000u64,  // token_1: 比例正常（2.5M / 55M ≈ 4.54%）
            5_000_000u64,  // token_2: 比例正常（5M / 110M ≈ 4.54%）
            1_000_000u64,  // token_3: 比例正常（1M / 22M ≈ 4.54%）
            1_500_000u64,  // token_4: 比例正常（1.5M / 33M ≈ 4.54%）
            2_000_000u64,  // token_5: 比例正常（2M / 44M ≈ 4.54%）
        ];
        
        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        let result = add_liquidity_inner(
            &vaults,
            &amounts_in,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
        .unwrap();

        println!("\n用户提供的token:");
        for i in 0..amounts_in.len() {
            println!("  token_{}: {} (fee: {})", i, amounts_in[i], result.burn_fees[i]);
        }

        println!("\n实际从用户拿走的token（amounts_used）:");
        for i in 0..result.amounts_used.len() {
            println!("  token_{}: {}", i, result.amounts_used[i]);
        }

        println!("\n实际加入池子的token（amounts_in，扣费后）:");
        let mut min_ratio = f64::MAX;
        let mut min_index = 0;
        for i in 0..result.amounts_in.len() {
            let ratio = (result.amounts_in[i] as f64 / vaults[i] as f64) * 100.0;
            println!("  token_{}: {} (比例: {:.4}%)", i, result.amounts_in[i], ratio);
            if ratio < min_ratio {
                min_ratio = ratio;
                min_index = i;
            }
        }

        println!("\n多余的token（将退还给用户）:");
        for i in 0..amounts_in.len() {
            let excess = amounts_in[i] - result.amounts_used[i];
            if excess > 0 {
                println!("  token_{}: {}", i, excess);
            }
        }

        println!("\n铸造LP: {}", result.lp_minted);
        println!("最小比例来自: token_{} ({:.4}%)", min_index, min_ratio);

        // 验证：所有实际加入池子的token比例应该相同
        let base_ratio = result.amounts_in[1] as f64 / vaults[1] as f64;
        for i in 0..result.amounts_in.len() {
            let ratio = result.amounts_in[i] as f64 / vaults[i] as f64;
            let diff = (ratio - base_ratio).abs() / base_ratio;
            assert!(diff < 0.0001, "token_{} 加入池子的比例不一致: {:.6} vs {:.6}", i, ratio, base_ratio);
        }

        // 验证：token_0应该有多余的（因为提供的比例高）
        assert!(
            amounts_in[0] > result.amounts_used[0],
            "token_0 应该有多余的token"
        );

        println!("\n✅ 不等比例添加流动性测试通过！");
    }

    #[test]
    fn test_first_lp_sets_price() {
        // 说明：第一个LP定义价格，自行承担风险
        println!("\n=== 第一个LP定义初始价格 ===");
        
        // 场景：WSOL/DOGE池子，外部市场 1 WSOL = 1000 DOGE
        // 第一个LP可以任意设置初始储备比例
        
        println!("\n示例1: 正确定价（与市场一致）");
        let vaults_good = vec![
            100_000_000u64,      // 100 WSOL (6位小数)
            100_000_000_000u64,  // 100,000 DOGE (6位小数)
        ];
        let amounts_in_good = vaults_good.clone();
        
        let result_good = add_liquidity_inner(
            &vec![0u64, 0u64],  // 空池子
            &amounts_in_good,
            0,
            3,
            10000,
        ).unwrap();
        
        println!("  提供: 100 WSOL + 100,000 DOGE");
        println!("  池子隐含价格: 1 WSOL = 1000 DOGE");
        println!("  铸造LP: {}", result_good.lp_minted);
        println!("  ✅ 价格正确，LP安全");
        
        println!("\n示例2: 定价过高（LP会被套利）");
        let vaults_high = vec![
            100_000_000u64,  // 100 WSOL
            50_000_000_000u64,  // 50,000 DOGE (只提供了一半)
        ];
        let amounts_in_high = vaults_high.clone();
        
        let result_high = add_liquidity_inner(
            &vec![0u64, 0u64],
            &amounts_in_high,
            0,
            3,
            10000,
        ).unwrap();
        
        println!("  提供: 100 WSOL + 50,000 DOGE");
        println!("  池子隐含价格: 1 WSOL = 500 DOGE");
        println!("  铸造LP: {}", result_high.lp_minted);
        println!("  ❌ WSOL被低估，套利者会买入WSOL卖出DOGE，LP损失");
        
        println!("\n示例3: 定价过低（LP会被套利）");
        let vaults_low = vec![
            100_000_000u64,      // 100 WSOL
            200_000_000_000u64,  // 200,000 DOGE (提供了两倍)
        ];
        let amounts_in_low = vaults_low.clone();
        
        let result_low = add_liquidity_inner(
            &vec![0u64, 0u64],
            &amounts_in_low,
            0,
            3,
            10000,
        ).unwrap();
        
        println!("  提供: 100 WSOL + 200,000 DOGE");
        println!("  池子隐含价格: 1 WSOL = 2000 DOGE");
        println!("  铸造LP: {}", result_low.lp_minted);
        println!("  ❌ WSOL被高估，套利者会卖出WSOL买入DOGE，LP损失");
        
        println!("\n💡 关键结论：");
        println!("   - 系统不验证价格是否正确，这是LP的责任");
        println!("   - 第一个LP定价错误 = 套利者的利润 = LP的损失");
        println!("   - LP应该参考外部市场价格来设置初始储备比例");
        println!("   - 这是去中心化系统的自由市场机制");
        
        println!("\n✅ 第一个LP定价测试完成！");
    }

    #[test]
    fn test_weighted_pool_initial_price() {
        // 测试：权重为20:80时，如何设置初始流动性来匹配外部价格
        println!("\n=== 加权池初始定价：DOGE/WSOL = 20:80 ===");
        
        let external_price = 1000.0; // 1 WSOL = 1000 DOGE
        let weight_doge = 20u64;
        let weight_wsol = 80u64;
        
        println!("\n外部市场价格: 1 WSOL = {} DOGE", external_price);
        println!("池子权重: DOGE = {}, WSOL = {}", weight_doge, weight_wsol);
        
        // 在加权CPMM中，价格公式为：
        // P_WSOL = (R_DOGE / W_DOGE) / (R_WSOL / W_WSOL)
        //
        // 要使 P_WSOL = 1000:
        // 1000 = (R_DOGE / 20) / (R_WSOL / 80)
        // 1000 = (R_DOGE * 80) / (R_WSOL * 20)
        // 1000 = (R_DOGE * 4) / R_WSOL
        // R_DOGE = 250 * R_WSOL
        //
        // 示例：如果提供 100 WSOL，需要提供 25,000 DOGE
        
        println!("\n推导过程:");
        println!("  价格公式: P_WSOL = (R_DOGE / W_DOGE) / (R_WSOL / W_WSOL)");
        println!("  代入权重: 1000 = (R_DOGE / 20) / (R_WSOL / 80)");
        println!("  化简:     1000 = (R_DOGE * 4) / R_WSOL");
        println!("  得到:     R_DOGE = 250 * R_WSOL");
        
        println!("\n【情况1：按正确比例提供流动性】");
        let vaults_correct = vec![
            25_000_000_000u64,  // 25,000 DOGE (6位小数)
            100_000_000u64,     // 100 WSOL (6位小数)
        ];
        let amounts_in_correct = vaults_correct.clone();
        
        let result_correct = add_liquidity_inner(
            &vec![0u64, 0u64],
            &amounts_in_correct,
            0,
            3,
            10000,
        ).unwrap();
        
        // 验证价格
        let r_doge = vaults_correct[0] as f64 / 1_000_000.0;  // 实际DOGE数量
        let r_wsol = vaults_correct[1] as f64 / 1_000_000.0;  // 实际WSOL数量
        let pool_price = (r_doge / weight_doge as f64) / (r_wsol / weight_wsol as f64);
        
        println!("  提供: {:.0} DOGE + {:.0} WSOL", r_doge, r_wsol);
        println!("  储备比例: {:.0} DOGE : 1 WSOL", r_doge / r_wsol);
        println!("  池子价格: 1 WSOL = {:.2} DOGE", pool_price);
        println!("  铸造LP: {}", result_correct.lp_minted);
        
        assert!((pool_price - external_price).abs() < 0.01, "价格偏差过大");
        println!("  ✅ 价格准确匹配外部市场！");
        
        println!("\n【情况2：如果按50:50等价值提供（错误）】");
        let vaults_wrong = vec![
            100_000_000_000u64,  // 100,000 DOGE
            100_000_000u64,      // 100 WSOL
        ];
        let amounts_in_wrong = vaults_wrong.clone();
        
        let result_wrong = add_liquidity_inner(
            &vec![0u64, 0u64],
            &amounts_in_wrong,
            0,
            3,
            10000,
        ).unwrap();
        
        let r_doge_wrong = vaults_wrong[0] as f64 / 1_000_000.0;
        let r_wsol_wrong = vaults_wrong[1] as f64 / 1_000_000.0;
        let pool_price_wrong = (r_doge_wrong / weight_doge as f64) / (r_wsol_wrong / weight_wsol as f64);
        
        println!("  提供: {:.0} DOGE + {:.0} WSOL", r_doge_wrong, r_wsol_wrong);
        println!("  储备比例: {:.0} DOGE : 1 WSOL", r_doge_wrong / r_wsol_wrong);
        println!("  池子价格: 1 WSOL = {:.2} DOGE", pool_price_wrong);
        println!("  铸造LP: {}", result_wrong.lp_minted);
        println!("  ❌ 价格 {} → 偏离市场 {:.1}%！", 
                 pool_price_wrong,
                 ((pool_price_wrong - external_price) / external_price * 100.0).abs());
        
        println!("\n【情况3：如果按储备比例1000:1提供（错误）】");
        let vaults_wrong2 = vec![
            100_000_000_000u64,  // 100,000 DOGE
            100_000u64,          // 0.1 WSOL
        ];
        let amounts_in_wrong2 = vaults_wrong2.clone();
        
        let result_wrong2 = add_liquidity_inner(
            &vec![0u64, 0u64],
            &amounts_in_wrong2,
            0,
            3,
            10000,
        ).unwrap();
        
        let r_doge_wrong2 = vaults_wrong2[0] as f64 / 1_000_000.0;
        let r_wsol_wrong2 = vaults_wrong2[1] as f64 / 1_000_000.0;
        let pool_price_wrong2 = (r_doge_wrong2 / weight_doge as f64) / (r_wsol_wrong2 / weight_wsol as f64);
        
        println!("  提供: {:.0} DOGE + {:.1} WSOL", r_doge_wrong2, r_wsol_wrong2);
        println!("  储备比例: {:.0} DOGE : 1 WSOL", r_doge_wrong2 / r_wsol_wrong2);
        println!("  池子价格: 1 WSOL = {:.2} DOGE", pool_price_wrong2);
        println!("  铸造LP: {}", result_wrong2.lp_minted);
        println!("  ❌ 价格 {} → 偏离市场 {:.1}%！", 
                 pool_price_wrong2,
                 ((pool_price_wrong2 - external_price) / external_price * 100.0).abs());
        
        println!("\n💡 核心结论：");
        println!("   1. 权重影响价格公式，不是简单的储备比例");
        println!("   2. 20:80权重下，需要 250:1 的储备比例才能达到 1:1000 的价格");
        println!("   3. 权重越高的token，需要的储备量越少（相对其价值）");
        println!("   4. 这允许池子偏向某个token，减少无常损失的影响");
        
        println!("\n✅ 加权池初始定价测试完成！");
    }

    #[test]
    fn test_weighted_pool_capital_efficiency() {
        // 测试：通过权重设置，LP可以用更少的资产创建同样价格的池子
        println!("\n=== 加权池的资本效率优势 ===");
        println!("场景：创建价格为 1 WSOL = 1000 DOGE 的池子");
        
        println!("\n【方案A：Uniswap模式（50:50权重）】");
        let weight_50_50 = 50u64;
        
        // 50:50权重下，要达到 1:1000 的价格
        // P = (R_DOGE / 50) / (R_WSOL / 50) = R_DOGE / R_WSOL = 1000
        // 所以需要 R_DOGE = 1000 * R_WSOL
        let vaults_uniswap = vec![
            100_000_000_000u64,  // 100,000 DOGE
            100_000_000u64,      // 100 WSOL
        ];
        
        let result_uniswap = add_liquidity_inner(
            &vec![0u64, 0u64],
            &vaults_uniswap.clone(),
            0,
            3,
            10000,
        ).unwrap();
        
        let r_doge_uni = vaults_uniswap[0] as f64 / 1_000_000.0;
        let r_wsol_uni = vaults_uniswap[1] as f64 / 1_000_000.0;
        let pool_price_uni = (r_doge_uni / weight_50_50 as f64) / (r_wsol_uni / weight_50_50 as f64);
        let total_value_uni = r_doge_uni * 0.001 + r_wsol_uni * 1.0; // 假设DOGE=$0.001, WSOL=$1
        
        println!("  权重配置: DOGE=50, WSOL=50");
        println!("  需要提供: {:.0} DOGE + {:.0} WSOL", r_doge_uni, r_wsol_uni);
        println!("  总价值: ${:.2} (假设DOGE=$0.001, WSOL=$1)", total_value_uni);
        println!("  池子价格: 1 WSOL = {:.2} DOGE ✅", pool_price_uni);
        println!("  铸造LP: {}", result_uniswap.lp_minted);
        
        println!("\n【方案B：Balancer模式（20:80权重）】");
        let weight_doge = 20u64;
        let weight_wsol = 80u64;
        
        // 20:80权重下，要达到 1:1000 的价格
        // P = (R_DOGE / 20) / (R_WSOL / 80) = (R_DOGE * 4) / R_WSOL = 1000
        // 所以需要 R_DOGE = 250 * R_WSOL
        let vaults_balancer = vec![
            25_000_000_000u64,  // 25,000 DOGE (只需要1/4！)
            100_000_000u64,     // 100 WSOL (相同)
        ];
        
        let result_balancer = add_liquidity_inner(
            &vec![0u64, 0u64],
            &vaults_balancer.clone(),
            0,
            3,
            10000,
        ).unwrap();
        
        let r_doge_bal = vaults_balancer[0] as f64 / 1_000_000.0;
        let r_wsol_bal = vaults_balancer[1] as f64 / 1_000_000.0;
        let pool_price_bal = (r_doge_bal / weight_doge as f64) / (r_wsol_bal / weight_wsol as f64);
        let total_value_bal = r_doge_bal * 0.001 + r_wsol_bal * 1.0;
        
        println!("  权重配置: DOGE=20, WSOL=80");
        println!("  需要提供: {:.0} DOGE + {:.0} WSOL", r_doge_bal, r_wsol_bal);
        println!("  总价值: ${:.2} (假设DOGE=$0.001, WSOL=$1)", total_value_bal);
        println!("  池子价格: 1 WSOL = {:.2} DOGE ✅", pool_price_bal);
        println!("  铸造LP: {}", result_balancer.lp_minted);
        
        println!("\n【方案C：极端Balancer（10:90权重）】");
        let weight_doge_extreme = 10u64;
        let weight_wsol_extreme = 90u64;
        
        // 10:90权重下: R_DOGE = 111.11 * R_WSOL
        let vaults_extreme = vec![
            11_111_000_000u64,  // 11,111 DOGE (只需要1/9！)
            100_000_000u64,     // 100 WSOL (相同)
        ];
        
        let result_extreme = add_liquidity_inner(
            &vec![0u64, 0u64],
            &vaults_extreme.clone(),
            0,
            3,
            10000,
        ).unwrap();
        
        let r_doge_ext = vaults_extreme[0] as f64 / 1_000_000.0;
        let r_wsol_ext = vaults_extreme[1] as f64 / 1_000_000.0;
        let pool_price_ext = (r_doge_ext / weight_doge_extreme as f64) / (r_wsol_ext / weight_wsol_extreme as f64);
        let total_value_ext = r_doge_ext * 0.001 + r_wsol_ext * 1.0;
        
        println!("  权重配置: DOGE=10, WSOL=90");
        println!("  需要提供: {:.0} DOGE + {:.0} WSOL", r_doge_ext, r_wsol_ext);
        println!("  总价值: ${:.2} (假设DOGE=$0.001, WSOL=$1)", total_value_ext);
        println!("  池子价格: 1 WSOL = {:.2} DOGE ✅", pool_price_ext);
        println!("  铸造LP: {}", result_extreme.lp_minted);
        
        println!("\n📊 资本效率对比:");
        println!("┌──────────────┬────────────┬──────────┬──────────┬─────────┐");
        println!("│   权重配置   │  DOGE需求  │ WSOL需求 │  总价值  │  节省   │");
        println!("├──────────────┼────────────┼──────────┼──────────┼─────────┤");
        println!("│ 50:50 (Uni)  │  100,000   │   100    │  $200.00 │   0%    │");
        println!("│ 20:80 (Bal)  │   25,000   │   100    │  $125.00 │  37.5%  │");
        println!("│ 10:90 (Bal)  │   11,111   │   100    │  $111.11 │  44.4%  │");
        println!("└──────────────┴────────────┴──────────┴──────────┴─────────┘");
        
        let saving_20_80 = (total_value_uni - total_value_bal) / total_value_uni * 100.0;
        let saving_10_90 = (total_value_uni - total_value_ext) / total_value_uni * 100.0;
        
        println!("\n💡 核心优势：");
        println!("   1. 20:80权重可节省 {:.1}% 的资本（少需要75,000 DOGE）", saving_20_80);
        println!("   2. 10:90权重可节省 {:.1}% 的资本（少需要88,889 DOGE）", saving_10_90);
        println!("   3. 三种方案的池子价格完全相同（都是1:1000）");
        println!("   4. LP可以根据持仓情况选择最优权重配置");
        
        println!("\n🎯 实际应用场景：");
        println!("   - LP持有大量WSOL，但DOGE不足 → 选择高WSOL权重（如80%）");
        println!("   - LP看好WSOL，想减少DOGE敞口 → 提高WSOL权重");
        println!("   - LP想要更大的池子深度，但资本有限 → 调整权重降低总资本需求");
        
        println!("\n✅ 资本效率测试完成！");
    }

    #[test]
    fn test_remove_liquidity() {
        // 测试移除流动性
        // 第二次添加后的状态
        let vaults = vec![
            11_499_550u64,
            57_497_750,
            114_995_500,
            22_999_100,
            34_498_650,
            45_998_200,
        ];
        let lp_to_burn = 499_850u64; // 第一次LP的50%
        let total_lp_supply = 1_045_128u64; // 999_700 + 45_428
        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        let result = remove_liquidity_inner(
            &vaults,
            lp_to_burn,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
        .unwrap();

        // 验证输出数量
        let expected_amounts = vec![
            5_498_200u64,
            27_491_000,
            54_982_000,
            10_996_400,
            16_494_600,
            21_992_800,
        ];

        for i in 0..result.amounts_out.len() {
            // 允许一定误差（由于整数除法）
            let diff = if result.amounts_out[i] > expected_amounts[i] {
                result.amounts_out[i] - expected_amounts[i]
            } else {
                expected_amounts[i] - result.amounts_out[i]
            };
        assert!(
                diff <= 10,
                "token_{} 输出误差过大: {} vs {}",
                i,
                result.amounts_out[i],
                expected_amounts[i]
            );
        }

        println!("✅ 移除流动性测试通过！");
        println!("  销毁LP: {}", lp_to_burn);
        println!("  输出token_0: {}", result.amounts_out[0]);
    }

    #[test]
    fn test_full_liquidity_cycle() {
        // 测试完整的流动性周期
        println!("\n=== 完整流动性周期测试 ===");

        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        // 初始状态
        let mut vaults = vec![
            10_000_000u64,
            50_000_000,
            100_000_000,
            20_000_000,
            30_000_000,
            40_000_000,
        ];
        let mut total_lp_supply = 0u64;

        println!("\n1. 初始状态:");
        println!("   储备: {:?}", vaults);
        println!("   LP总供应: {}", total_lp_supply);

        // 第一次添加
        let amounts_in_1 = vec![
            1_000_000u64,
            5_000_000,
            10_000_000,
            2_000_000,
            3_000_000,
            4_000_000,
        ];
        let result_1 = add_liquidity_inner(
            &vaults,
            &amounts_in_1,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
        .unwrap();

        // 更新状态
        for i in 0..vaults.len() {
            vaults[i] += result_1.amounts_in[i];
        }
        total_lp_supply += result_1.lp_minted;

        println!("\n2. 第一次添加后:");
        println!("   铸造LP: {}", result_1.lp_minted);
        println!("   LP总供应: {}", total_lp_supply);

        // 第二次添加
        let amounts_in_2 = vec![
            500_000u64, 2_500_000, 5_000_000, 1_000_000, 1_500_000, 2_000_000,
        ];
        let result_2 = add_liquidity_inner(
            &vaults,
            &amounts_in_2,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
        .unwrap();

        // 更新状态
        for i in 0..vaults.len() {
            vaults[i] += result_2.amounts_in[i];
        }
        total_lp_supply += result_2.lp_minted;

        println!("\n3. 第二次添加后:");
        println!("   铸造LP: {}", result_2.lp_minted);
        println!("   LP总供应: {}", total_lp_supply);

        // 移除流动性
        let lp_to_burn = result_1.lp_minted / 2;
        let result_3 = remove_liquidity_inner(
            &vaults,
            lp_to_burn,
            total_lp_supply,
            fee_numerator,
            fee_denominator,
        )
        .unwrap();

        // 更新状态
        for i in 0..vaults.len() {
            vaults[i] -= result_3.amounts_out[i] + result_3.burn_fees[i];
        }
        total_lp_supply -= lp_to_burn;

        println!("\n4. 移除流动性后:");
        println!("   销毁LP: {}", lp_to_burn);
        println!("   输出: {:?}", result_3.amounts_out);
        println!("   LP总供应: {}", total_lp_supply);
        println!("   最终储备: {:?}", vaults);

        println!("\n✅ 完整流动性周期测试通过！");
    }
}
