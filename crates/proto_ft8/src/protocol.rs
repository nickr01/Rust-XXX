use transport::rustxxx::*;

pub type U71 = u128;
pub type U28 = u32;

pub const FT8: Protocol = Protocol::new(
    Secs(0.16),
    Secs(15.0),
    true,
    BitCount(3),
    SymbolCount(58),
    SymbolCount(79),             // Total channel symbols (FT8_NS + FT8_ND)
    SymbolCount(7),     // sync group length
    RepeatCount(3),        // Number of sync groups
    SymbolCount(0),
    SymbolCount(36),    // Offset between sync groups
    [3, 1, 4, 0, 6, 5, 2],    //　Costas array
    BitCount(174),        // Number of bits in the encoded message (payload with LDPC checksum bits)
    BitCount(91),         // Number of payload bits (including CRC)
    [0, 1, 3, 2, 5, 6, 4, 7],
    [0, 1, 3, 2, 6, 4, 5, 7],
    CrcParams::new(BitCount(5),BitMap(0x2757), BitCount(14), 0, 0),
    2.0f32,
    SymbolCount(1),
);

// as per FT4/FT8 doc QEX  July/August 2020 - Franke et al
pub const C28_DE: U28 = 0;
pub const C28_QRZ: U28 = 1;
pub const C28_CQ: U28 = 2;

pub const C28_CQ_DDD: U28 = 3;
pub const C28_CQ_DDD_BLOCK: U28 = 1002-3+1;
pub const C28_CQ_DDD_UNDEF:U28 = C28_CQ_DDD + C28_CQ_DDD_BLOCK;
pub const C28_CQ_DDD_UNDEF_BLOCK: U28 = 1004 - C28_CQ_DDD_UNDEF;

pub const C28_CQ_A: U28 = C28_CQ_DDD + C28_CQ_DDD_BLOCK + C28_CQ_DDD_UNDEF_BLOCK;
pub const C28_CQ_A_BLOCK: U28 = 1029-1004+1;
pub const C28_CQ_A_UNDEF: U28 = C28_CQ_A + C28_CQ_A_BLOCK;
pub const C28_CQ_A_UNDEF_BLOCK: U28 = 1031 - C28_CQ_A_UNDEF;

pub const C28_CQ_AA: U28 = C28_CQ_A_UNDEF + C28_CQ_A_UNDEF_BLOCK;
pub const C28_CQ_AA_BLOCK: U28 = 1731-1031+1;
pub const C28_CQ_AA_UNDEF: U28 = C28_CQ_AA + C28_CQ_AA_BLOCK;
pub const C28_CQ_AA_UNDEF_BLOCK: U28 = 1760 - C28_CQ_AA_UNDEF;

pub const C28_CQ_AAA: U28 = C28_CQ_AA_UNDEF + C28_CQ_AA_UNDEF_BLOCK;
pub const C28_CQ_AAA_BLOCK: U28 = 20685 - 1760 + 1;
pub const C28_CQ_AAA_UNDEF: U28 = C28_CQ_AAA + C28_CQ_AAA_BLOCK;
pub const C28_CQ_AAA_UNDEF_BLOCK: U28 = 21443 - C28_CQ_AAA_UNDEF;

pub const C28_CQ_AAAA: U28 = C28_CQ_AAA_UNDEF + C28_CQ_AAA_UNDEF_BLOCK;
pub const C28_CQ_AAAA_BLOCK: U28 = 532443 - 21443 + 1;
pub const C28_CQ_AAAA_UNDEF: U28 = C28_CQ_AAAA + C28_CQ_AAAA_BLOCK;
pub const C28_CQ_AAAA_UNDEF_BLOCK: U28 = 2063592 - C28_CQ_AAAA_UNDEF;

pub const C28_HASH_CALL: U28 = C28_CQ_AAAA_UNDEF + C28_CQ_AAAA_UNDEF_BLOCK;
pub const C28_HASH_CALL_BLOCK: U28 = 0x400000; // 4194304;
pub const C28_HASH_CALL_UNDEF: U28 = C28_HASH_CALL + C28_HASH_CALL_BLOCK;
pub const C28_HASH_CALL_UNDEF_BLOCK: U28 = 6257896 - C28_HASH_CALL_UNDEF;

pub const C28_STD_CALLS: U28 = C28_HASH_CALL_UNDEF + C28_HASH_CALL_UNDEF_BLOCK; // 6257896;

pub const U28_MAX: U28 = u32::MAX; // Hmmm - this may be wrong!

// pub const BIT_i3_2: usize = 76;
// pub const BIT_i3_0: usize = 74;
// pub const BIT_n3_2: usize = 73;
// pub const BIT_n3_0: usize = 71;

// const NTOKENS: U71 = 2063592;
// const MAX22: U71 = 0x400000; // 4194304;
// const test1: U71 = NTOKENS + MAX22;

pub const MAXGRID4: u16 = 32400;

pub const FT8_MESSAGE_BITS: usize = 71;
pub const A71_BYTES: usize = (FT8_MESSAGE_BITS + 1) / 8; // 9;
pub type A71 = [u8; A71_BYTES];



