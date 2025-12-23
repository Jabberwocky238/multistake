use crate::error::ErrorCode;
use crate::math::i256::I256;
use crate::math::logexpmath::LogExpMath;
use crate::state::AnySwapPool;
use anchor_lang::prelude::*;
use primitive_types::U256;

pub struct SwapResult {
    pub burn_fees: Vec<u64>,
    pub amounts: Vec<u64>,
}

pub trait SwapProtocol {
    // 使用权重恒定乘积公式: a^wa * b^wb * c^wc * ... = K
    // 公式: sum(weight_i * ln(vault_i)) = constant
    fn swap<'info>(
        &self,
        // 当前index的token是否属于输入项
        is_in: &[bool],
        // 当前index的token的允许误差，当输入时，此为上界，当输出时，此为下界
        amount_tolerance: &[u64],
        // 用户提供的token账户
        user_vaults_amount: &[u64],
        // 池子中的token账户
        token_vaults_amount: &[u64],
        // weight
        weights: &[u64],
        // 费率分子
        fee_numerator: u64,
        // 费率分母
        fee_denominator: u64,
        // 返回合法操作的token数，输入值index为用户提供，输出值index为池中的token
    ) -> Result<SwapResult>;
}

/// 实现多token交换，使用权重恒定乘积公式（对数形式）
///
/// 公式: sum(weight_i * ln(vault_i)) = constant
///
/// 对于多进多出的交换：
/// 1. 计算交换前的恒定乘积（对数形式）
/// 2. 应用费率，计算实际输入（扣除费率后）
/// 3. 计算输入token的增量
/// 4. 对于前n-1个输出token，使用最小输出要求
/// 5. 对于最后一个输出token，根据恒定乘积公式计算
fn swap_inner<'info>(
    is_in: &[bool],
    amount_tolerance: &[u64],
    user_vaults_amount: &[u64],
    token_vaults_amount: &[u64],
    weights: &[u64],
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<SwapResult> {
    let token_count = is_in.len();
    require!(
        amount_tolerance.len() == token_count,
        ErrorCode::InvalidTokenCount
    );
    require!(
        user_vaults_amount.len() == token_count,
        ErrorCode::InvalidTokenCount
    );
    require!(
        token_vaults_amount.len() == token_count,
        ErrorCode::InvalidTokenCount
    );
    require!(weights.len() == token_count, ErrorCode::InvalidTokenCount);
    
    // LogExpMath期望18位小数精度
    // vault需要放大18位（因为ln需要18位精度输入）
    // weight保持原始值（作为系数）
    let constant_before = weights
        .iter()
        .enumerate()
        .map(|(i, weight)| {
            let vault_before = token_vaults_amount[i];
            // 将vault放大18位
            let vault_before_u256 = U256::from(vault_before) * U256::from(1_000_000_000_000_000_000u64);
            let vault_before_i256 = I256::try_from(vault_before_u256).unwrap();
            // weight不放大，ln返回1e18精度
            let weight_i256 = I256::from(*weight);
            let delta = weight_i256 * LogExpMath::ln(vault_before_i256).unwrap();
            delta
        })
        .sum::<I256>();
    let mut vaults_after = token_vaults_amount.iter().map(|x| *x).collect::<Vec<u64>>();

    // 初始化输出数组
    let mut outputs = vec![0u64; token_count];
    
    // 计算费用：对输入token，从amount_tolerance中扣除费用
    // 先检查用户余额
    for (i, &tolerance) in amount_tolerance.iter().enumerate() {
        if is_in[i] {
            require!(
                user_vaults_amount[i] >= tolerance,
                ErrorCode::InsufficientTokenAmount
            );
        }
    }
    
    let burn_fees: Vec<u64> = amount_tolerance
        .iter()
        .enumerate()
        .map(|(i, &tolerance)| {
            if !is_in[i] {
                return 0;
            }
            // 计算费用
            let amount_u256 = U256::from(tolerance);
            let fee_amount = (amount_u256 * fee_numerator) / fee_denominator;
            fee_amount.as_u64()
        })
        .collect::<Vec<u64>>();

    // amounts_in_after_fee是扣除费用后的实际输入金额
    let amounts_in_after_fee = amount_tolerance
        .iter()
        .zip(burn_fees.iter())
        .enumerate()
        .filter(|(i, _)| is_in[*i])
        .map(|(_, (&tolerance, &burn_fee))| U256::from(tolerance - burn_fee))
        .collect::<Vec<U256>>();
    // 输入token的索引和池子储备
    let amounts_in_index: Vec<usize> = is_in
        .iter()
        .enumerate()
        .filter(|(_, &is_in)| is_in)
        .map(|(i, _)| i)
        .collect();
    
    let amounts_in_pool: Vec<U256> = amounts_in_index
        .iter()
        .map(|&i| U256::from(token_vaults_amount[i]))
        .collect();
    
    // 输出token的索引、最小输出要求和池子储备
    let amounts_out_index: Vec<usize> = is_in
        .iter()
        .enumerate()
        .filter(|(_, &is_in)| !is_in)
        .map(|(i, _)| i)
        .collect();
    
    let amounts_out_min: Vec<U256> = amounts_out_index
        .iter()
        .map(|&i| U256::from(amount_tolerance[i]))
        .collect();
    
    let amounts_out_pool: Vec<U256> = amounts_out_index
        .iter()
        .map(|&i| U256::from(token_vaults_amount[i]))
        .collect();

    let mut delta_sum = I256::ZERO;

    // 处理输入token（储备增加）
    // delta = weights[i] * ln(vaults_after[i])
    for (i, (&amount_after_fee, &amount_in_pool)) in amounts_in_after_fee
        .iter()
        .zip(amounts_in_pool.iter())
        .enumerate()
    {
        let idx = amounts_in_index[i];
        let vault_after = amount_after_fee + amount_in_pool;
        vaults_after[idx] = vault_after.as_u64();
        // 将vault放大18位
        let vault_after_u256 = vault_after * U256::from(1_000_000_000_000_000_000u64);
        let vault_after_i256 = I256::try_from(vault_after_u256)?;
        // weight不放大
        let weight_i256 = I256::from(weights[idx]);
        let delta = weight_i256 * LogExpMath::ln(vault_after_i256)?;
        delta_sum = delta_sum + delta;
        // outputs记录扣除费用后的实际输入
        outputs[idx] = amount_after_fee.as_u64();
    }

    // 处理输出token（除了最后一个）
    // delta = weights[i] * ln(vaults_after[i])
    for (i, (&amount_out_min, &amount_out_pool)) in amounts_out_min
        .iter()
        .zip(amounts_out_pool.iter())
        .enumerate()
    {
        if i == amounts_out_min.len() - 1 {
            continue;
        }
        let idx = amounts_out_index[i];
        // 检查池子储备是否足够
        require!(
            amount_out_pool >= amount_out_min,
            ErrorCode::InsufficientLiquidity
        );
        let vault_after = amount_out_pool - amount_out_min;
        // 将vault放大18位
        let vault_after_u256 = vault_after * U256::from(1_000_000_000_000_000_000u64);
        let vault_after_i256 = I256::try_from(vault_after_u256)?;
        // weight不放大
        let weight_i256 = I256::from(weights[idx]);
        let delta = weight_i256 * LogExpMath::ln(vault_after_i256)?;
        delta_sum = delta_sum + delta;
        // outputs记录实际输出（vault减少量）
        let actual_output = amount_out_pool.as_u64() - vault_after.as_u64();
        outputs[idx] = actual_output;
    }

    // 计算最后一个输出 token 应该的值
    let last_idx = amounts_out_index[amounts_out_index.len() - 1];
    let last_weight = I256::from(weights[last_idx]);

    // last_delta + sum(weights[i] * ln(vaults_after[i])) = constant_before
    let last_delta = constant_before - delta_sum;
    // last_delta除以weight得到ln值（1e18精度）
    let last_ln_vault_after = last_delta / last_weight;
    // exp返回的是vault*1e18，需要除以1e18得到原始vault
    let last_should_be_18 = LogExpMath::exp(last_ln_vault_after)?;
    let last_should_be = last_should_be_18 / I256::from(1_000_000_000_000_000_000u64);

    #[cfg(test)]
    {
        println!("=== 调试信息 ===");
        println!("last_idx: {}", last_idx);
        println!("constant_before: {:?}", constant_before);
        println!("delta_sum: {:?}", delta_sum);
        println!("last_delta: {:?}", last_delta);
        println!("last_weight: {:?}", last_weight);
        println!("last_ln_vault_after: {:?}", last_ln_vault_after);
        println!("last_should_be_18: {:?}", last_should_be_18);
        println!("last_should_be: {:?}", last_should_be);
        println!(
            "token_vaults_amount[{}]: {}",
            last_idx, token_vaults_amount[last_idx]
        );
        println!(
            "vaults_after[{}] (计算后): {}",
            last_idx,
            last_should_be.as_u64()
        );
    }

    require!(last_should_be > I256::ZERO, ErrorCode::MathOverflow);
    require!(last_should_be <= I256::MAX, ErrorCode::MathOverflow);
    require!(
        last_should_be.as_u64() <= token_vaults_amount[last_idx],
        ErrorCode::InsufficientLiquidity
    );
    vaults_after[last_idx] = last_should_be.as_u64();
    let last_amount_out = token_vaults_amount[last_idx] - vaults_after[last_idx];
    outputs[last_idx] = last_amount_out;

    Ok(SwapResult {
        burn_fees: burn_fees,
        amounts: outputs,
    })
}

impl SwapProtocol for AnySwapPool {
    fn swap<'info>(
        &self,
        is_in: &[bool],
        amount_tolerance: &[u64],
        user_vaults_amount: &[u64],
        token_vaults_amount: &[u64],
        weights: &[u64],
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<SwapResult> {
        swap_inner(
            is_in,
            amount_tolerance,
            user_vaults_amount,
            token_vaults_amount,
            weights,
            fee_numerator,
            fee_denominator,
        )
    }
}

// cargo test --manifest-path programs/anyswap/Cargo.toml test_swap_6_tokens_3in_2out --lib -- --nocapture
#[cfg(test)]
mod tests {
    use super::*;

    // 创建一个简单的结构体来实现SwapProtocol，用于测试
    struct TestSwap;

    impl SwapProtocol for TestSwap {
        fn swap<'info>(
            &self,
            is_in: &[bool],
            amount_tolerance: &[u64],
            user_vaults_amount: &[u64],
            token_vaults_amount: &[u64],
            weights: &[u64],
            fee_numerator: u64,
            fee_denominator: u64,
        ) -> Result<SwapResult> {
            swap_inner(
                is_in,
                amount_tolerance,
                user_vaults_amount,
                token_vaults_amount,
                weights,
                fee_numerator,
                fee_denominator,
            )
        }
    }

    #[test]
    fn test_swap_6_tokens_3in_2out() {
        // 测试用例2：6 token swap，3进2出（权重不同、储备不同、输出不同）
        // 参数与Python测试用例2相同
        let swap_impl = TestSwap;
        // vaults = [1e7, 5e7, 1e8, 2e7, 3e7, 4e7]  # 初始储备：各不相同
        // weights = [20, 40, 80, 30, 50, 60]  # 权重：各不相同
        // amounts_in = [1e5, 2e5, 1.5e5]  # 输入：token 0, 1, 2，数量不同
        // 初始储备：各不相同
        let token_vaults_amount = vec![
            10_000_000u64,  // token 0: 10M
            50_000_000u64,  // token 1: 50M
            100_000_000u64, // token 2: 100M
            20_000_000u64,  // token 3: 20M
            30_000_000u64,  // token 4: 30M
            40_000_000u64,  // token 5: 40M
        ];

        // 权重：各不相同（原始值，函数内部会放大18位）
        let weights = vec![20u64, 40u64, 80u64, 30u64, 50u64, 60u64];

        // 输入：token 0, 1, 2，数量不同（匹配Python测试：1e5, 2e5, 1.5e5）
        let user_vaults_amount = vec![
            1_000_000u64, // token 0: 用户有1M
            2_000_000u64, // token 1: 用户有2M
            1_500_000u64, // token 2: 用户有1.5M
            1_200_000u64, // token 3: 输出
            1_000_000u64, // token 4: 输出
            0u64,         // token 5: 不是输入
        ];

        // is_in: token 0, 1, 2 是输入，token 3, 4 是输出
        let is_in = vec![true, true, true, false, false, false];

        // amount_tolerance:
        // 对于输入token (0, 1, 2): 最大输入量
        // 对于输出token (3, 4): 最小输出量
        // 根据Python测试用例2，token3期望输出120,000，token4期望输出178,162.417339
        // 最小输出 = 期望输出的99%
        let amount_tolerance = vec![
            100_000u64, // token 0: 最大输入100K
            200_000u64, // token 1: 最大输入200K
            150_000u64, // token 2: 最大输入150K
            118_800u64, // token 3: 最小输出118,800 (120K * 0.99)
            176_380u64, // token 4: 最小输出176,380 (178,162 * 0.99)
        ];

        // 执行交换
        // 费率：万分之三
        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        // 只有前五个token参与交换
        let result = swap_impl.swap(
            &is_in[..5],
            &amount_tolerance,
            &user_vaults_amount[..5],
            &token_vaults_amount[..5],
            &weights[..5],
            fee_numerator,
            fee_denominator,
        );

        match &result {
            Ok(SwapResult { amounts, burn_fees }) => {
                println!("交换成功，结果: {:?}, {:?}", amounts, burn_fees);
            }
            Err(e) => {
                panic!("交换失败: {:?}", e);
            }
        }
        let SwapResult { amounts, burn_fees } = result.unwrap();

        // 验证 burn_fees: 输入token有费用，输出token费用为0
        assert_eq!(burn_fees.len(), amounts.len());
        for i in 0..amounts.len() {
            println!("burn_fees[{}]: {}", i, burn_fees[i]);
            println!("amounts[{}]: {}", i, amounts[i]);
            if i < 3 {
                // 输入token：有费用（基于amount_tolerance）
                assert_eq!(
                    burn_fees[i],
                    amount_tolerance[i] * fee_numerator / fee_denominator,
                    "token_{} 的burn_fee应该等于输入量（tolerance）的费率", i
                );
            } else {
                // 输出token：费用为0
                assert_eq!(burn_fees[i], 0u64, "token_{} 是输出token，burn_fee应该为0", i);
            }
        }

        // 验证输入token的实际输入（扣除费率后）
        // 费率是万分之三，所以 100,000 * (1 - 0.0003) = 99,970
        assert_eq!(
            amounts[0], 99_970u64,
            "token_0 实际输入应该是扣除费率后的值"
        );
        assert_eq!(
            amounts[1], 199_940u64,
            "token_1 实际输入应该是扣除费率后的值"
        );
        assert_eq!(
            amounts[2], 149_955u64,
            "token_2 实际输入应该是扣除费率后的值"
        );

        // 验证输出token满足最小要求
        assert!(
            amounts[3] >= amount_tolerance[3],
            "token_3 输出应该满足最小要求"
        );
        assert!(
            amounts[4] >= amount_tolerance[4],
            "token_4 输出应该满足最小要求"
        );

        println!("\n✅ 测试用例2通过！");
    }

    #[test]
    fn test_swap_30_tokens_10in_10out() {
        // 测试用例3：30 token swap，10进10出（包含大额交易，权重9位小数）
        let swap_impl = TestSwap;
        
        // 池子储备（30个token）
        let token_vaults_amount: Vec<u64> = vec![
            10000000000u64, 50000000000u64, 200000000000u64, 30000000000u64, 80000000000u64,  // tokens 0-4
            150000000000u64, 40000000000u64, 60000000000u64, 90000000000u64, 120000000000u64,  // tokens 5-9
            25000000000u64, 70000000000u64, 110000000000u64, 35000000000u64, 55000000000u64,  // tokens 10-14
            85000000000u64, 130000000000u64, 45000000000u64, 65000000000u64, 95000000000u64,  // tokens 15-19
            20000000000u64, 40000000000u64, 70000000000u64, 100000000000u64, 30000000000u64,  // tokens 20-24
            50000000000u64, 80000000000u64, 110000000000u64, 40000000000u64, 60000000000u64,  // tokens 25-29
        ];

        // 权重（30个token，9位小数精度，已乘以1e9）
        let weights: Vec<u64> = vec![
            10000000000u64, 15000000000u64, 25000000000u64, 12000000000u64, 18000000000u64,  // tokens 0-4
            30000000000u64, 14000000000u64, 20000000000u64, 22000000000u64, 28000000000u64,  // tokens 5-9
            16000000000u64, 24000000000u64, 26000000000u64, 13000000000u64, 19000000000u64,  // tokens 10-14
            23000000000u64, 29000000000u64, 11000000000u64, 21000000000u64, 27000000000u64,  // tokens 15-19
            17000000000u64, 15000000000u64, 20000000000u64, 25000000000u64, 12000000000u64,  // tokens 20-24
            18000000000u64, 22000000000u64, 28000000000u64, 14000000000u64, 16000000000u64,  // tokens 25-29
        ];

        // 输入token的tolerance（10个，索引0-9）
        // 注意：token 0 是 5万亿，超过了10e12的要求
        let amounts_in_tolerance: Vec<u64> = vec![
            5000000000000u64,  // token 0: 5万亿（超过10e12）
            1000000000u64,     // token 1: 10亿
            5000000000u64,     // token 2: 50亿
            2000000000u64,     // token 3: 20亿
            8000000000u64,     // token 4: 80亿
            3000000000u64,     // token 5: 30亿
            1500000000u64,     // token 6: 15亿
            6000000000u64,     // token 7: 60亿
            4000000000u64,     // token 8: 40亿
            2500000000u64,     // token 9: 25亿
        ];

        // 输出token的最小要求（10个，索引10-19，99%期望值）
        let amounts_out_min: Vec<u64> = vec![
            495000000u64,       // token 10
            990000000u64,       // token 11
            1485000000u64,      // token 12
            594000000u64,       // token 13
            792000000u64,       // token 14
            1188000000u64,      // token 15
            1980000000u64,      // token 16
            693000000u64,       // token 17
            891000000u64,       // token 18
            86259345270u64,     // token 19
        ];

        // 构造user_vaults_amount和amount_tolerance
        // 前10个是输入token，接下来10个是输出token，后10个不参与
        let mut user_vaults_amount = vec![0u64; 30];
        let mut amount_tolerance = vec![0u64; 30];
        
        // 输入token：用户账户有足够余额
        for i in 0..10 {
            user_vaults_amount[i] = amounts_in_tolerance[i] * 2; // 余额是输入量的2倍
            amount_tolerance[i] = amounts_in_tolerance[i];
        }
        
        // 输出token：设置最小输出要求
        for i in 0..10 {
            user_vaults_amount[10 + i] = 0; // 输出token用户余额不重要
            amount_tolerance[10 + i] = amounts_out_min[i];
        }

        // is_in: 前10个是输入，接下来10个是输出，后10个不参与
        let mut is_in = vec![false; 30];
        for i in 0..10 {
            is_in[i] = true;
        }

        // 执行交换
        let fee_numerator = 3u64;
        let fee_denominator = 10000u64;

        let result = swap_impl.swap(
            &is_in[..20], // 只有前20个token参与交换
            &amount_tolerance[..20],
            &user_vaults_amount[..20],
            &token_vaults_amount[..20],
            &weights[..20],
            fee_numerator,
            fee_denominator,
        );

        match &result {
            Ok(SwapResult { amounts, burn_fees }) => {
                println!("\n✅ 交换成功！");
                println!("输入token（扣费后）:");
                for i in 0..10 {
                    println!("  token_{}: {} (fee: {})", i, amounts[i], burn_fees[i]);
                }
                println!("\n输出token:");
                for i in 0..10 {
                    println!("  token_{}: {} (min: {})", 
                        10+i, amounts[10+i], amounts_out_min[i]);
                }
            }
            Err(e) => {
                panic!("交换失败: {:?}", e);
            }
        }
        
        let SwapResult { amounts, burn_fees } = result.unwrap();

        // 验证burn_fees
        for i in 0..20 {
            if i < 10 {
                // 输入token有费用
                let expected_fee = amounts_in_tolerance[i] * fee_numerator / fee_denominator;
                assert_eq!(burn_fees[i], expected_fee, "token_{} 的burn_fee不正确", i);
            } else {
                // 输出token费用为0
                assert_eq!(burn_fees[i], 0u64, "token_{} 是输出token，burn_fee应该为0", i);
            }
        }

        // 验证输出满足最小要求
        for i in 0..10 {
            assert!(
                amounts[10 + i] >= amounts_out_min[i],
                "token_{} 输出 {} 小于最小要求 {}",
                10 + i,
                amounts[10 + i],
                amounts_out_min[i]
            );
        }

        // 验证最后一个输出token接近预期值（允许一定误差）
        let expected_last_output = 87130651788u64;
        let actual_last_output = amounts[19];
        let diff = if actual_last_output > expected_last_output {
            actual_last_output - expected_last_output
        } else {
            expected_last_output - actual_last_output
        };
        let tolerance = expected_last_output / 2000; // 0.05%误差
        println!("tolerance: {} diff: {}", tolerance, diff);
        assert!(
            diff <= tolerance,
            "token_19 输出 {} 与预期 {} 差距过大（差值: {}）",
            actual_last_output,
            expected_last_output,
            diff
        );

        println!("\n✅ 测试用例3通过：30 token swap，10进10出，包含大额交易（5万亿）！");
    }
}
