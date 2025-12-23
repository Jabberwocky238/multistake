use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("代币顺序无效：token_0 的地址必须小于 token_1")]
    InvalidTokenOrder,
    #[msg("LP Mint 地址不匹配")]
    InvalidLpMint,
    #[msg("数学运算溢出")]
    MathOverflow,
    #[msg("流动性不足")]
    InsufficientLiquidity,
    #[msg("代币数量不足")]
    InsufficientTokenAmount,
    #[msg("储备量不足")]
    InsufficientReserves,
    #[msg("输出数量不足（滑点过大）")]
    InsufficientOutputAmount,
    #[msg("无效的代币 mint 地址")]
    InvalidTokenMint,
    #[msg("无效的 token 数量")]
    InvalidTokenCount,
    #[msg("已达到最大 token 数量限制")]
    MaxTokensReached,
    #[msg("无效的 token 索引")]
    InvalidTokenIndex,
    #[msg("不能交换相同的 token")]
    SameTokenSwap,
    #[msg("无效的管理员")]
    InvalidAdmin,
}

