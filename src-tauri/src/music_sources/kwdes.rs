//! kwDES 加密算法 —— 酷我音乐 mobi.s 接口的 query 加密
//! 移植自 UnblockNeteaseMusic/server 的 kwDES.js
//! 参考: https://github.com/UnblockNeteaseMusic/server/blob/enhanced/src/kwDES.js

// EXPANSION (E 扩展置换, 64个元素, 32->48位)
const ARRAY_E: [i32; 64] = [
    31, 0, 1, 2, 3, 4, -1, -1, 3, 4, 5, 6, 7, 8, -1, -1,
    7, 8, 9, 10, 11, 12, -1, -1, 11, 12, 13, 14, 15, 16, -1, -1,
    15, 16, 17, 18, 19, 20, -1, -1, 19, 20, 21, 22, 23, 24, -1, -1,
    23, 24, 25, 26, 27, 28, -1, -1, 27, 28, 29, 30, 31, 30, -1, -1,
];

// INITIAL_PERMUTATION (IP 初始置换, 64个元素)
const ARRAY_IP: [i32; 64] = [
    57, 49, 41, 33, 25, 17, 9, 1,
    59, 51, 43, 35, 27, 19, 11, 3,
    61, 53, 45, 37, 29, 21, 13, 5,
    63, 55, 47, 39, 31, 23, 15, 7,
    56, 48, 40, 32, 24, 16, 8, 0,
    58, 50, 42, 34, 26, 18, 10, 2,
    60, 52, 44, 36, 28, 20, 12, 4,
    62, 54, 46, 38, 30, 22, 14, 6,
];

// INVERSE_PERMUTATION (IP-1 逆置换, 64个元素)
const ARRAY_IP_1: [i32; 64] = [
    39, 7, 47, 15, 55, 23, 63, 31,
    38, 6, 46, 14, 54, 22, 62, 30,
    37, 5, 45, 13, 53, 21, 61, 29,
    36, 4, 44, 12, 52, 20, 60, 28,
    35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26,
    33, 1, 41, 9, 49, 17, 57, 25,
    32, 0, 40, 8, 48, 16, 56, 24,
];

// ROTATES (循环移位表, 16个元素)
const ARRAY_LS: [u32; 16] = [
    1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1,
];

// 移位掩码
const ARRAY_LS_MASK: [u64; 3] = [0, 0x100001, 0x300003];

// PERMUTATION (P 置换, 32个元素)
const ARRAY_P: [i32; 32] = [
    15, 6, 19, 20, 28, 11, 27, 16,
    0, 14, 22, 25, 4, 17, 30, 9,
    1, 7, 23, 13, 31, 26, 2, 8,
    18, 12, 29, 5, 21, 10, 3, 24,
];

// PERMUTED_CHOICE1 (PC-1, 56个元素)
const ARRAY_PC_1: [i32; 56] = [
    56, 48, 40, 32, 24, 16, 8,
    0, 57, 49, 41, 33, 25, 17,
    9, 1, 58, 50, 42, 34, 26,
    18, 10, 2, 59, 51, 43, 35,
    62, 54, 46, 38, 30, 22, 14,
    6, 61, 53, 45, 37, 29, 21,
    13, 5, 60, 52, 44, 36, 28,
    20, 12, 4, 27, 19, 11, 3,
];

// PERMUTED_CHOICE2 (PC-2, 64个元素, -1表示跳过)
const ARRAY_PC_2: [i32; 64] = [
    13, 16, 10, 23, 0, 4, -1, -1,
    2, 27, 14, 5, 20, 9, -1, -1,
    22, 18, 11, 3, 25, 7, -1, -1,
    15, 6, 26, 19, 12, 1, -1, -1,
    40, 51, 30, 36, 46, 54, -1, -1,
    29, 39, 50, 44, 32, 47, -1, -1,
    43, 48, 38, 55, 33, 52, -1, -1,
    45, 41, 49, 35, 28, 31, -1, -1,
];

// S-Box (8个子表, 每个64个元素)
const MATRIX_NS_BOX: [[u8; 64]; 8] = [
    [
        14, 4, 3, 15, 2, 13, 5, 3, 13, 14, 6, 9, 11, 2, 0, 5, 4, 1, 10, 12, 15,
        6, 9, 10, 1, 8, 12, 7, 8, 11, 7, 0, 0, 15, 10, 5, 14, 4, 9, 10, 7, 8,
        12, 3, 13, 1, 3, 6, 15, 12, 6, 11, 2, 9, 5, 0, 4, 2, 11, 14, 1, 7, 8,
        13,
    ],
    [
        15, 0, 9, 5, 6, 10, 12, 9, 8, 7, 2, 12, 3, 13, 5, 2, 1, 14, 7, 8, 11, 4,
        0, 3, 14, 11, 13, 6, 4, 1, 10, 15, 3, 13, 12, 11, 15, 3, 6, 0, 4, 10, 1,
        7, 8, 4, 11, 14, 13, 8, 0, 6, 2, 15, 9, 5, 7, 1, 10, 12, 14, 2, 5, 9,
    ],
    [
        10, 13, 1, 11, 6, 8, 11, 5, 9, 4, 12, 2, 15, 3, 2, 14, 0, 6, 13, 1, 3,
        15, 4, 10, 14, 9, 7, 12, 5, 0, 8, 7, 13, 1, 2, 4, 3, 6, 12, 11, 0, 13,
        5, 14, 6, 8, 15, 2, 7, 10, 8, 15, 4, 9, 11, 5, 9, 0, 14, 3, 10, 7, 1,
        12,
    ],
    [
        7, 10, 1, 15, 0, 12, 11, 5, 14, 9, 8, 3, 9, 7, 4, 8, 13, 6, 2, 1, 6, 11,
        12, 2, 3, 0, 5, 14, 10, 13, 15, 4, 13, 3, 4, 9, 6, 10, 1, 12, 11, 0, 2,
        5, 0, 13, 14, 2, 8, 15, 7, 4, 15, 1, 10, 7, 5, 6, 12, 11, 3, 8, 9, 14,
    ],
    [
        2, 4, 8, 15, 7, 10, 13, 6, 4, 1, 3, 12, 11, 7, 14, 0, 12, 2, 5, 9, 10,
        13, 0, 3, 1, 11, 15, 5, 6, 8, 9, 14, 14, 11, 5, 6, 4, 1, 3, 10, 2, 12,
        15, 0, 13, 2, 8, 5, 11, 8, 0, 15, 7, 14, 9, 4, 12, 7, 10, 9, 1, 13, 6,
        3,
    ],
    [
        12, 9, 0, 7, 9, 2, 14, 1, 10, 15, 3, 4, 6, 12, 5, 11, 1, 14, 13, 0, 2,
        8, 7, 13, 15, 5, 4, 10, 8, 3, 11, 6, 10, 4, 6, 11, 7, 9, 0, 6, 4, 2, 13,
        1, 9, 15, 3, 8, 15, 3, 1, 14, 12, 5, 11, 0, 2, 12, 14, 7, 5, 10, 8, 13,
    ],
    [
        4, 1, 3, 10, 15, 12, 5, 0, 2, 11, 9, 6, 8, 7, 6, 9, 11, 4, 12, 15, 0, 3,
        10, 5, 14, 13, 7, 8, 13, 14, 1, 2, 13, 6, 14, 9, 4, 1, 2, 14, 11, 13, 5,
        0, 1, 10, 8, 3, 0, 11, 3, 5, 9, 4, 15, 2, 7, 8, 12, 15, 10, 7, 6, 12,
    ],
    [
        13, 7, 10, 0, 6, 9, 5, 15, 8, 4, 3, 10, 11, 14, 12, 5, 2, 11, 9, 6, 15,
        12, 0, 3, 4, 1, 14, 13, 1, 2, 7, 8, 1, 2, 12, 15, 10, 4, 0, 3, 13, 14,
        6, 9, 7, 8, 9, 6, 15, 1, 5, 12, 3, 10, 14, 5, 8, 7, 11, 0, 4, 13, 2, 11,
    ],
];

/// 按位置换：从 l 中按 arr 的索引取位，放到结果的对应位置
fn bit_transform(arr: &[i32], l: u64) -> u64 {
    let mut result: u64 = 0;
    for (i, &idx) in arr.iter().enumerate() {
        if idx < 0 {
            continue;
        }
        if l & (1u64 << idx as u64) != 0 {
            result |= 1u64 << i as u64;
        }
    }
    result
}

/// 生成 16 个子密钥
fn sub_keys(l: u64, mode: i32) -> [u64; 16] {
    let mut l2 = bit_transform(&ARRAY_PC_1, l);
    let mut longs = [0u64; 16];

    for i in 0..16 {
        let ls = ARRAY_LS[i] as u32;
        let mask = ARRAY_LS_MASK[ARRAY_LS[i] as usize];

        // (l2 & mask) << (28 - ls) | (l2 & !mask) >> ls
        let part1 = (l2 & mask).wrapping_shl(28 - ls);
        let part2 = (l2 & !mask) >> ls;
        l2 = part1 | part2;

        longs[i] = bit_transform(&ARRAY_PC_2, l2);
    }

    // 解密模式：反转子密钥顺序
    if mode == 1 {
        for j in 0..8 {
            longs.swap(j, 15 - j);
        }
    }

    longs
}

/// DES 加密一个 64 位块
fn des64(subkeys: &[u64; 16], l: u64) -> u64 {
    let mut out = bit_transform(&ARRAY_IP, l);
    let mut p0 = out & 0xffffffff; // 低 32 位 (pSource[0])
    let mut p1 = out >> 32; // 高 32 位 (pSource[1])

    for i in 0..16 {
        let mut r = p1;
        r = bit_transform(&ARRAY_E, r);
        r ^= subkeys[i];

        // 取 8 个 8 位组
        let pr: [u64; 8] = [
            (r >> 0) & 255,
            (r >> 8) & 255,
            (r >> 16) & 255,
            (r >> 24) & 255,
            (r >> 32) & 255,
            (r >> 40) & 255,
            (r >> 48) & 255,
            (r >> 56) & 255,
        ];

        // S-Box 替换（反向遍历，每次左移 4 位）
        let mut s_out: u64 = 0;
        for sbi in (0..8).rev() {
            s_out = (s_out << 4) | MATRIX_NS_BOX[sbi][pr[sbi] as usize] as u64;
        }

        r = bit_transform(&ARRAY_P, s_out);
        let temp = p0;
        p0 = p1;
        p1 = temp ^ r;
    }

    // pSource.reverse() 后: pSource[0] = p1, pSource[1] = p0
    // out = (pSource[1] << 32) | pSource[0] = (p0 << 32) | p1
    out = (p0 << 32) | (p1 & 0xffffffff);
    out = bit_transform(&ARRAY_IP_1, out);
    out
}

/// 加密/解密消息
fn crypt(msg: &[u8], key: &[u8], mode: i32) -> Vec<u8> {
    // 处理密钥块：将 8 字节密钥转为 u64
    let mut l: u64 = 0;
    for i in 0..8 {
        l |= (key[i] as u64) << (i * 8);
    }

    let subkeys = sub_keys(l, mode);

    let j = msg.len() / 8;
    let mut result = Vec::new();

    // 处理完整块
    for m in 0..j {
        let mut block: u64 = 0;
        for n in 0..8 {
            block |= (msg[n + m * 8] as u64) << (n * 8);
        }
        let encrypted = des64(&subkeys, block);
        for n in 0..8 {
            result.push((encrypted >> (n * 8)) as u8);
        }
    }

    // 处理剩余字节（加密模式追加完整块）
    let remainder = msg.len() % 8;
    if remainder > 0 || mode == 0 {
        let mut block: u64 = 0;
        let start = j * 8;
        for i in 0..remainder {
            block |= (msg[start + i] as u64) << (i * 8);
        }
        let encrypted = des64(&subkeys, block);
        for n in 0..8 {
            result.push((encrypted >> (n * 8)) as u8);
        }
    }

    result
}

const SECRET_KEY: &[u8] = b"ylzsxkwm";

/// 加密 query 字符串并返回 base64 编码
pub fn encrypt_query(query: &str) -> String {
    use base64::Engine;
    let encrypted = crypt(query.as_bytes(), SECRET_KEY, 0);
    base64::engine::general_purpose::STANDARD.encode(&encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 kwDES 加密能正常执行且输出 base64
    #[test]
    fn test_encrypt_query_basic() {
        let query = "corp=kuwo&source=kwplayer_ar_5.1.0.0_B_jiakong_vh.apk&p2p=1&type=convert_url2&sig=0&format=mp3&rid=321260769";
        let result = encrypt_query(query);
        assert!(!result.is_empty());
        // base64 字符串只包含合法字符
        assert!(result.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
        println!("kwDES encrypt_query result: {}", result);
    }

    /// 真实接口测试：用 kwDES 加密的 query 请求 kuwo mobi.s 获取 VIP 歌曲 URL
    /// 手动运行: cargo test -- --ignored --nocapture
    #[tokio::test]
    #[ignore]
    async fn test_kuwo_mobi_with_kwdes() {
        let rid = "321260769"; // 邓紫棋「唯一」VIP 歌曲
        let query = format!(
            "corp=kuwo&source=kwplayer_ar_5.1.0.0_B_jiakong_vh.apk&p2p=1&type=convert_url2&sig=0&format=mp3&rid={}",
            rid
        );
        let encrypted = encrypt_query(&query);
        let url = format!("http://mobi.kuwo.cn/mobi.s?f=kuwo&q={}", encrypted);

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header("user-agent", "okhttp/3.10.0")
            .send()
            .await
            .expect("请求应成功");

        let text = resp.text().await.expect("读取响应应成功");
        println!("kuwo mobi.s 响应 (前 500 字符): {}", &text[..text.len().min(500)]);

        // 响应中应包含 http URL
        assert!(text.contains("http"), "响应应包含 URL: {}", text);
    }
}
