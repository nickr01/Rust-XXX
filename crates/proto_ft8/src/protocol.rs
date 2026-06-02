use transport::rustxxx::*;

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

// TODO: something more elegant with slices
pub const CALL_A0: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ+-./?";
pub const CALL_A0_LEN: u32 = CALL_A0.len() as u32;

pub const CALL_A1: &str = " 0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const CALL_A1_LEN: u32 = CALL_A1.len() as u32;

pub const CALL_A2: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const CALL_A2_LEN: u32 = CALL_A2.len() as u32;

pub const CALL_A3: &str = "0123456789";
pub const CALL_A3_LEN: u32 = CALL_A3.len() as u32;

pub const CALL_A4: &str = " ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const CALL_A4_LEN: u32 = CALL_A4.len() as u32;

// as per FT4/FT8 doc QEX  July/August 2020 - Franke et al
pub const C28_DE: u32 = 0;
pub const C28_QRZ: u32 = 1;
pub const C28_CQ: u32 = 2;

pub const C28_CQ_DDD: u32 = 3;
pub const C28_CQ_DDD_BLOCK: u32 = 1002-3+1;
pub const C28_CQ_DDD_UNDEF:u32 = C28_CQ_DDD + C28_CQ_DDD_BLOCK;
pub const C28_CQ_DDD_UNDEF_BLOCK: u32 = 1004 - C28_CQ_DDD_UNDEF;

pub const C28_CQ_A: u32 = C28_CQ_DDD + C28_CQ_DDD_BLOCK + C28_CQ_DDD_UNDEF_BLOCK;
pub const C28_CQ_A_BLOCK: u32 = 1029-1004+1;
pub const C28_CQ_A_UNDEF: u32 = C28_CQ_A + C28_CQ_A_BLOCK;
pub const C28_CQ_A_UNDEF_BLOCK: u32 = 1031 - C28_CQ_A_UNDEF;

pub const C28_CQ_AA: u32 = C28_CQ_A_UNDEF + C28_CQ_A_UNDEF_BLOCK;
pub const C28_CQ_AA_BLOCK: u32 = 1731-1031+1;
pub const C28_CQ_AA_UNDEF: u32 = C28_CQ_AA + C28_CQ_AA_BLOCK;
pub const C28_CQ_AA_UNDEF_BLOCK: u32 = 1760 - C28_CQ_AA_UNDEF;

pub const C28_CQ_AAA: u32 = C28_CQ_AA_UNDEF + C28_CQ_AA_UNDEF_BLOCK;
pub const C28_CQ_AAA_BLOCK: u32 = 20685 - 1760 + 1;
pub const C28_CQ_AAA_UNDEF: u32 = C28_CQ_AAA + C28_CQ_AAA_BLOCK;
pub const C28_CQ_AAA_UNDEF_BLOCK: u32 = 21443 - C28_CQ_AAA_UNDEF;

pub const C28_CQ_AAAA: u32 = C28_CQ_AAA_UNDEF + C28_CQ_AAA_UNDEF_BLOCK;
pub const C28_CQ_AAAA_BLOCK: u32 = 532443 - 21443 + 1;
pub const C28_CQ_AAAA_UNDEF: u32 = C28_CQ_AAAA + C28_CQ_AAAA_BLOCK;
pub const C28_CQ_AAAA_UNDEF_BLOCK: u32 = 2063592 - C28_CQ_AAAA_UNDEF;

pub const C28_HASH_CALL: u32 = C28_CQ_AAAA_UNDEF + C28_CQ_AAAA_UNDEF_BLOCK;
pub const C28_HASH_CALL_BLOCK: u32 = 0x400000; // 4194304;
pub const C28_HASH_CALL_UNDEF: u32 = C28_HASH_CALL + C28_HASH_CALL_BLOCK;
pub const C28_HASH_CALL_UNDEF_BLOCK: u32 = 6257896 - C28_HASH_CALL_UNDEF;

pub const C28_STD_CALLS: u32 = C28_HASH_CALL_UNDEF + C28_HASH_CALL_UNDEF_BLOCK; // 6257896;

// pub const BIT_i3_2: usize = 76;
// pub const BIT_i3_0: usize = 74;
// pub const BIT_n3_2: usize = 73;
// pub const BIT_n3_0: usize = 71;

// const NTOKENS: u32 = 2063592;
// const MAX22: u32 = 0x400000; // 4194304;
// const test1: u32 = NTOKENS + MAX22;

pub const MAXGRID4: u16 = 32400;

