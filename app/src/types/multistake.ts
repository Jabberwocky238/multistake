/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/multistake.json`.
 */
export type Multistake = {
  "address": "2mgSDKAjDo8fQN6oms6YzczHhyeYEJunTzxjQgegYADf",
  "metadata": {
    "name": "multistake",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "addTokenToPool",
      "docs": [
        "添加质押类型到 Pool",
        "自动创建 LP mint，使用 increment_count 作为 seed",
        "权重默认 10^8"
      ],
      "discriminator": [
        35,
        121,
        233,
        111,
        213,
        155,
        197,
        192
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "docs": [
            "Pool authority PDA - LP mint 的 authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  110,
                  121,
                  115,
                  119,
                  97,
                  112,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "lpMint",
          "docs": [
            "LP mint - 自动创建，权限归属于 pool_authority",
            "使用 increment_count 作为 seed 确保唯一性（只增不减）"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "admin",
          "docs": [
            "Pool 管理员 - 必须签名"
          ],
          "signer": true
        },
        {
          "name": "payer",
          "docs": [
            "支付创建 LP mint 的费用"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "createPool",
      "docs": [
        "创建 Pool（PDA）"
      ],
      "discriminator": [
        233,
        146,
        209,
        142,
        207,
        104,
        64,
        188
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "docs": [
            "Pool authority PDA - 用于管理 pool vault"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  110,
                  121,
                  115,
                  119,
                  97,
                  112,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "mainTokenMint",
          "docs": [
            "主币的 Mint 账户 - Pool 对应的币种"
          ]
        },
        {
          "name": "poolVault",
          "docs": [
            "Pool 的主币 Vault - 存储所有质押的主币"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "admin",
          "docs": [
            "Pool 管理员 - 用于所有操作的权限控制"
          ],
          "signer": true
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "feeNumerator",
          "type": "u64"
        },
        {
          "name": "feeDenominator",
          "type": "u64"
        }
      ]
    },
    {
      "name": "modifyTokenWeight",
      "docs": [
        "修改 token 的 weight"
      ],
      "discriminator": [
        239,
        78,
        110,
        20,
        65,
        97,
        193,
        233
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "admin",
          "docs": [
            "Pool 管理员 - 必须签名所有操作"
          ],
          "signer": true
        }
      ],
      "args": [
        {
          "name": "newWeights",
          "type": {
            "vec": "u64"
          }
        }
      ]
    },
    {
      "name": "removeTokenFromPool",
      "docs": [
        "从 MultiStake Pool 移除 token"
      ],
      "discriminator": [
        104,
        117,
        82,
        78,
        25,
        2,
        55,
        49
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "lpMint",
          "docs": [
            "要移除的 LP mint 账户"
          ]
        },
        {
          "name": "admin",
          "docs": [
            "Pool 管理员 - 必须签名"
          ],
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "stake",
      "docs": [
        "质押主币，铸造 LP 凭证"
      ],
      "discriminator": [
        206,
        176,
        202,
        18,
        200,
        209,
        179,
        108
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "docs": [
            "Pool authority PDA - LP mint 的 authority"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  110,
                  121,
                  115,
                  119,
                  97,
                  112,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "poolVault",
          "docs": [
            "Pool 的主币 Vault"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "lpMint",
          "docs": [
            "LP mint - 对应的质押类型",
            "通过 pool.get_token() 验证地址是否匹配"
          ],
          "writable": true
        },
        {
          "name": "userMainToken",
          "docs": [
            "用户的主币账户（质押来源）"
          ],
          "writable": true
        },
        {
          "name": "userLpToken",
          "docs": [
            "用户的 LP 凭证账户（铸造目标）"
          ],
          "writable": true
        },
        {
          "name": "user",
          "docs": [
            "用户签名"
          ],
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "itemIndex",
          "type": "u16"
        },
        {
          "name": "stakeAmount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "unstake",
      "docs": [
        "销毁 LP 凭证，赎回主币"
      ],
      "discriminator": [
        90,
        95,
        107,
        42,
        205,
        124,
        50,
        225
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "docs": [
            "Pool authority PDA"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  97,
                  110,
                  121,
                  115,
                  119,
                  97,
                  112,
                  95,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "poolVault",
          "docs": [
            "Pool 的主币 Vault"
          ],
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
                  95,
                  118,
                  97,
                  117,
                  108,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "lpMint",
          "docs": [
            "LP mint - 对应的质押类型",
            "通过 pool.get_token() 验证地址是否匹配"
          ],
          "writable": true
        },
        {
          "name": "userLpToken",
          "docs": [
            "用户的 LP 凭证账户（销毁来源）"
          ],
          "writable": true
        },
        {
          "name": "userMainToken",
          "docs": [
            "用户的主币账户（赎回目标）"
          ],
          "writable": true
        },
        {
          "name": "user",
          "docs": [
            "用户签名"
          ],
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "itemIndex",
          "type": "u16"
        },
        {
          "name": "lpAmount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "pool",
      "discriminator": [
        241,
        154,
        109,
        4,
        17,
        177,
        109,
        188
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidTokenOrder",
      "msg": "代币顺序无效：token_0 的地址必须小于 token_1"
    },
    {
      "code": 6001,
      "name": "invalidLpMint",
      "msg": "LP Mint 地址不匹配"
    },
    {
      "code": 6002,
      "name": "mathOverflow",
      "msg": ""
    },
    {
      "code": 6003,
      "name": "insufficientLiquidity",
      "msg": ""
    },
    {
      "code": 6004,
      "name": "insufficientTokenAmount",
      "msg": ""
    },
    {
      "code": 6005,
      "name": "insufficientReserves",
      "msg": ""
    },
    {
      "code": 6006,
      "name": "insufficientOutputAmount",
      "msg": "输出数量不足（滑点过大）"
    },
    {
      "code": 6007,
      "name": "invalidTokenMint",
      "msg": "无效的代币 mint 地址"
    },
    {
      "code": 6008,
      "name": "invalidTokenCount",
      "msg": "无效的 token 数量"
    },
    {
      "code": 6009,
      "name": "maxTokensReached",
      "msg": "已达到最大 token 数量限制"
    },
    {
      "code": 6010,
      "name": "invalidTokenIndex",
      "msg": "无效的 token 索引"
    },
    {
      "code": 6011,
      "name": "sameTokenSwap",
      "msg": "不能交换相同的 token"
    },
    {
      "code": 6012,
      "name": "invalidAdmin",
      "msg": ""
    }
  ],
  "types": [
    {
      "name": "pool",
      "docs": [
        "单币质押池结构",
        "",
        "一个 Pool 对应一种主币，支持多种质押类型（items）",
        "使用 zero_copy 以避免栈溢出（大数组需要）"
      ],
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "tokenCount",
            "docs": [
              "实际使用的质押类型数量"
            ],
            "type": "u16"
          },
          {
            "name": "incrementCount",
            "docs": [
              "创建计数器 - 用于生成唯一的 LP mint seed，只增不减"
            ],
            "type": "u16"
          },
          {
            "name": "padding",
            "docs": [
              "填充字节（确保 8 字节对齐）"
            ],
            "type": {
              "array": [
                "u8",
                4
              ]
            }
          },
          {
            "name": "admin",
            "docs": [
              "Pool 管理员 - 用于所有操作的权限控制"
            ],
            "type": "pubkey"
          },
          {
            "name": "poolVault",
            "docs": [
              "Pool 的主币 Vault 账户 - 存储所有质押的主币"
            ],
            "type": "pubkey"
          },
          {
            "name": "poolMint",
            "docs": [
              "Pool 的主币 Mint 地址 - 该 Pool 对应的币种"
            ],
            "type": "pubkey"
          },
          {
            "name": "feeNumerator",
            "docs": [
              "手续费分子"
            ],
            "type": "u64"
          },
          {
            "name": "feeDenominator",
            "docs": [
              "手续费分母"
            ],
            "type": "u64"
          },
          {
            "name": "tokens",
            "docs": [
              "质押类型配置数组，最多支持 1024 种质押类型（固定大小）",
              "每个 item 记录一种质押类型的 LP mint、已发行量和权重"
            ],
            "type": {
              "array": [
                {
                  "defined": {
                    "name": "poolItem"
                  }
                },
                512
              ]
            }
          }
        ]
      }
    },
    {
      "name": "poolItem",
      "docs": [
        "质押类型配置项",
        "每个 item 记录一种质押类型的 LP mint、已发行量和权重",
        "用于单币质押系统，不同质押类型有不同的收益权重"
      ],
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "mintAccount",
            "docs": [
              "LP 凭证 Mint account 地址 - 用户质押后获得的凭证 token (32 bytes)"
            ],
            "type": "pubkey"
          },
          {
            "name": "mintAmount",
            "docs": [
              "已铸造的 LP 凭证数量 - 该质押类型的总发行量 (8 bytes)"
            ],
            "type": "u64"
          },
          {
            "name": "weight",
            "docs": [
              "权重 (weight) - 动态权重，由 admin 通过 oracle 修改 (8 bytes)",
              "影响 LP 凭证兑换主币的比率，weight 越高收益越好"
            ],
            "type": "u64"
          }
        ]
      }
    }
  ]
};
