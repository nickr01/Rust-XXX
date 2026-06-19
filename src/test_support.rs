use crate::types::*;

// this is a clone of FT8
pub const TEST_PROTOCOL: Protocol = Protocol::new(
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
    CrcParams::new(BitCount(5), BitMap(0x2757), BitCount(14), 0, 0),
    // crc_polynomial: BitMap(0x2757),   // CRC-14 polynomial without the leading (MSB) 1 0x2757 {8174,8174,18,18,4,4,2,2}
    // crc_width: BitCount(14),
    // crc_start: 0,
    // crc_xor: 0,
    2.0f32,
    SymbolCount(1),
);

pub const TEST_FT8_RUNTIME: Runtime = Runtime::new(
    // should be indep of bandwidth and freq_osr but not there yet
    Hz(6000.0),  // this is the real design layer - app layer can chose a portion often 250-2500
    RepeatCount(1), 
    BitCount(32),
    OverSampleMultiplier(4), // 4
    OverSampleMultiplier(2), // 2
    // detector_underload_divisor: RepeatCount(1), // 2 as per WB2FKO doc
    1.0, // 0.4, // 10,
    RepeatCount(1),
    RepeatCount(20),
    false, // true, 
    // subtracts: RepeatCount(1),
    WindowFunction::_Hann,  // Hann in the FT8_lib c code, or Blackman
);  

pub const TEST_FREQUENCY: Hz = Hz(1500.0);
