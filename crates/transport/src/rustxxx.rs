use std::sync::OnceLock;

use thiserror::Error;

// use crate::rx_streamed::StreamReceiver; // - for library level errors

pub const AUDIO_INPUT_BUFSIZE: usize = 2_usize.pow(21); // 22 for file // 20
pub const WATERFALL_BUF_SIZE: usize = 320; // This should be dynamic!!!  2_usize.pow(9);

pub const AUDIO_OUTPUT_BUFSIZE: usize = 2_usize.pow(20);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Hz(pub f32);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Secs(pub f32);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct BitCount(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct BitMap(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct ByteCount(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct SymbolCount(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct RepeatCount(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct OverSampleMultiplier(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct TimeIndex(pub usize);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct TimeStamp(pub u32);

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct FreqIndex(pub usize);

pub const fn bits2bytes(bits: BitCount) -> ByteCount {
    ByteCount(bits.0.div_ceil(8))
}

pub type AudioSampleBuffer = ringbuf::SharedRb<ringbuf::storage::Heap<f32>>; // SharedRb<ringbuf::storage::Heap<f32>>;
pub type AudioBufWriter = ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, true, false>; 
pub type AudioBufReader = ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, false, true>;
pub type ThreadedAudioReader = std::sync::Arc<std::sync::Mutex<ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, false, true>>>;
pub type ThreadedAudioWriter = std::sync::Arc<std::sync::Mutex<ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, true, false>>>;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct CrcParams {
    prepad: usize,  // 5 as per FT8 doc
    width: u8,      // rustxxx::FT8._crc_width().0 as u8,
    poly: u16,      // rustxxx::FT8._crc_polynomial().0 as u16,
    init: u16,      // rustxxx::FT8._crc_start() as u16,
    // refin: false,
    // refout: false,
    xorout: u16,    // rustxxx::FT8._crc_xor() as u16,
    check: u16,     // 0x0,
    residue: u16,   // 0x0
}

    // pub fn new     crc_polynomial: BitMap(0x2757),   // CRC-14 polynomial without the leading (MSB) 1 0x2757 {8174,8174,18,18,4,4,2,2}
    // crc_width: BitCount(14),
    // crc_start: 0,
    // crc_xor: 0, -> CrcParams {

impl CrcParams {
    pub const fn new (
        prepad: BitCount,
        poly: BitMap,
        width: BitCount,
        init: u16,
        xorout: u16,
        ) -> CrcParams {
        CrcParams {
            prepad: prepad.0,
            width: width.0 as u8,
            poly: poly.0 as u16,
            init,
            xorout,
            check: 0,
            residue: 0,
        }
    }

    pub const fn prepad(&self) -> usize {
        self.prepad
    }
    pub const fn width(&self) -> u8 {
        self.width
    }
    pub const fn poly(&self) -> u16 {
        self.poly
    }
    pub const fn init(&self) -> u16 {
        self.init
    }
    pub const fn xorout(&self) -> u16 {
        self.xorout
    }
    pub const fn check(&self) -> u16 {
        self.check
    }
    pub const fn residue(&self) -> u16 {
        self.residue
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Protocol {
    symbol_period: Secs,  // **
    slot_time: Secs,
    slot_time_locked: bool,
    token_bits: BitCount, // added for tests
    nd: SymbolCount,
    total_symbols_nn: SymbolCount, // Total channel symbols (FT8_NS + FT8_ND)
    length_sync: SymbolCount, // sync group length
    num_sync: RepeatCount,   // Number of sync groups
    length_ramp: SymbolCount,
    sync_offset: SymbolCount,   // Offset between sync groups
    costas_pattern: [u8; 7],    //　Costas array
    ldpc_n: BitCount,     // Number of bits in the encoded message (payload with LDPC checksum bits)
    ldpc_k: BitCount,     // Number of payload bits (including CRC)
    gray_map: [u8; 8],
    gray_rmap: [u8; 8],   // TODO work out how to autogen
    crc_params: CrcParams,
    symbol_bt: f32,
    ramp_symbols: SymbolCount,
}

impl Protocol {
    pub const fn new (
        symbol_period: Secs,  // **
        slot_time: Secs,
        slot_time_locked: bool,
        token_bits: BitCount, // added for tests
        nd: SymbolCount,
        total_symbols_nn: SymbolCount, // Total channel symbols (FT8_NS + FT8_ND)
        length_sync: SymbolCount, // sync group length
        num_sync: RepeatCount,   // Number of sync groups
        length_ramp: SymbolCount,
        sync_offset: SymbolCount,   // Offset between sync groups
        costas_pattern: [u8; 7],    //　Costas array
        ldpc_n: BitCount,     // Number of bits in the encoded message (payload with LDPC checksum bits)
        ldpc_k: BitCount,     // Number of payload bits (including CRC)
        gray_map: [u8; 8],
        gray_rmap: [u8; 8],   // TODO work out how to autogen
        crc_params: CrcParams,
        symbol_bt: f32,
        ramp_symbols: SymbolCount,
    ) -> Protocol {
        Protocol {
            symbol_period,
            slot_time,
            slot_time_locked,
            token_bits, // added for tests
            nd,
            total_symbols_nn, // Total channel symbols (FT8_NS + FT8_ND)
            length_sync, // sync group length
            num_sync,   // Number of sync groups
            length_ramp,
            sync_offset,   // Offset between sync groups
            costas_pattern,    //　Costas array
            ldpc_n,     // Number of bits in the encoded message (payload with LDPC checksum bits)
            ldpc_k,     // Number of payload bits (including CRC)
            gray_map,
            gray_rmap,   // TODO work out how to autogen
            crc_params,
            symbol_bt,
            ramp_symbols,
        }
    }
    pub const fn symbol_period(&self) -> Secs { self.symbol_period }
    pub const fn baud_rate(&self) -> Hz { Hz(1.0 / self.symbol_period().0) }
    pub const fn _slot_time(&self) -> Secs { self.slot_time }
    pub const fn _slot_time_locked(&self) -> bool { self.slot_time_locked }
    pub const fn _total_frame_time(&self) -> Secs { Secs(self.total_symbols_nn().0 as f32 * self.symbol_period().0 ) }
    pub const fn token_bits(&self) -> BitCount { self.token_bits }
    pub const fn token_tones(&self) -> RepeatCount { RepeatCount(1 << self.token_bits().0) }
    pub const fn nd(&self) -> SymbolCount { self.nd }
    pub const fn total_symbols_nn(&self) -> SymbolCount { self.total_symbols_nn }                 // Total channel symbols (FT8_NS + FT8_ND)
    pub const fn _ns(&self) -> SymbolCount { SymbolCount( self.length_sync.0 * self.num_sync.0 ) }
    pub const fn _length_sync(&self) -> SymbolCount { self.length_sync }        // sync group length
    pub const fn num_sync (&self) -> RepeatCount { self.num_sync }    // Number of sync groups
    pub const fn _length_ramp(&self) -> SymbolCount { self.length_ramp }
    pub const fn sync_offset(&self) -> SymbolCount { self.sync_offset }   // Offset between sync groups
    pub const fn costas_pattern(&self) -> [u8; 7] { self.costas_pattern }    //　Costas array
    pub const fn ldpc_n(&self) -> BitCount { self.ldpc_n }     // Number of bits in the encoded message (payload with LDPC checksum bits)
    pub const fn ldpc_k(&self) -> BitCount { self.ldpc_k}     // Number of payload bits (including CRC)
    pub const fn _ldpc_p(&self) -> BitCount { BitCount( self.ldpc_k().0 - self._crc_width().0 ) } // payload - no CRC    
    pub const fn gray_map(&self) -> [u8; 8] { self.gray_map }
    pub const fn _gray_rmap(&self) -> [u8; 8] { self.gray_rmap }

    pub const fn _crc_pad_bits(&self) -> BitCount { BitCount(self.crc_params.prepad()) }

    pub const fn _crc_polynomial(&self) -> BitMap { BitMap(self.crc_params.poly() as usize) }
    pub const fn _crc_width(&self) -> BitCount { BitCount(self.crc_params.width() as usize) }
    pub const fn _crc_start(&self) -> usize { self.crc_params.init() as usize }
    pub const fn _crc_xor(&self) -> usize { self.crc_params.xorout() as usize }

    pub const fn _symbol_bt(&self) -> f32 { self.symbol_bt }

    // pub const fn ramp_symbols(&self) -> SymbolCount { self.ramp_symbols }

    pub const fn ldpc_m(&self) -> BitCount { BitCount(self.ldpc_n().0 - self.ldpc_k().0) }

    pub const fn ldpc_n_bytes(&self) -> ByteCount { bits2bytes(self.ldpc_n()) }  // Number of whole bytes needed to store 174 bits (full message)
    pub const fn _ldpc_k_bytes(&self) -> ByteCount { bits2bytes(self.ldpc_k()) } // Number of whole bytes needed to store 91 bits (payload + CRC only)   
    pub const fn _ldpc_p_bytes(&self) -> ByteCount { bits2bytes(self._ldpc_p()) } // Number of whole bytes needed store payload only
    pub const fn _ldpc_p_padded_bytes(&self) -> ByteCount  { bits2bytes(BitCount(self._ldpc_p().0 + self._crc_pad_bits().0)) }
    // pub const fn ldpc_crc_pad_bytes(&self) -> ByteCount { ByteCount(self._ldpc_p_padded_bytes().0 - self._ldpc_p_bytes().0) }
}

// pub const _JS8A: Protocol = Protocol {
//     symbol_period: Secs(0.16),
//     slot_time: Secs(15.0),
//     slot_time_locked: true,
//     token_bits: BitCount(3),
//     nd: SymbolCount(58),
//     total_symbols_nn: SymbolCount(79),             // Total channel symbols (FT8_NS + FT8_ND)
//     length_sync: SymbolCount(7),    
//     num_sync: RepeatCount(3),        // Number of sync (Costas) groups
//     length_ramp: SymbolCount(0),
//     sync_offset: SymbolCount(36),    // Offset between sync groups
//     costas_pattern: [3, 1, 4, 0, 6, 5, 2],    //　Costas array
//     ldpc_n: BitCount(174),        // Number of bits in the encoded message (payload with LDPC checksum bits)
//     ldpc_k: BitCount(87),         // Number of payload bits (including CRC)
//     gray_map: [0, 1, 3, 2, 5, 6, 4, 7],
//     gray_rmap: [0, 1, 3, 2, 6, 4, 5, 7],
//     crc_calc_pad: BitCount(0),
//     crc_polynomial: BitMap(0xc06),   // CRC-14 polynomial without the leading (MSB) 1 
//     crc_width: BitCount(12),
//     crc_start: 0,
//     crc_xor: 0,
//     symbol_bt: 2.0f32,
//     ramp_symbols: SymbolCount(1),
// };

// // CRC choices see https://users.ece.cmu.edu/~koopman/crc/

// // Intended Licklider transport layer
// // Initially an FT8 clone but will need to be made incompatible by tweak of CRC params
// pub const _LT8A: Protocol = Protocol {
//     symbol_period: Secs(0.16),
//     slot_time: Secs(15.0),
//     slot_time_locked: true,
//     token_bits: BitCount(3),
//     nd: SymbolCount(58),
//     total_symbols_nn: SymbolCount(79),             // Total channel symbols (FT8_NS + FT8_ND)
//     length_sync: SymbolCount(7),     // sync group length
//     num_sync: RepeatCount(3),        // Number of sync groups
//     length_ramp: SymbolCount(0),
//     sync_offset: SymbolCount(36),    // Offset between sync groups
//     costas_pattern: [3, 1, 4, 0, 6, 5, 2],    //　Costas array
//     ldpc_n: BitCount(174),        // Number of bits in the encoded message (payload with LDPC checksum bits)
//     ldpc_k: BitCount(91),         // Number of payload bits (including CRC)
//     gray_map:  [0, 1, 3, 2, 5, 6, 4, 7],
//     gray_rmap: [0, 1, 3, 2, 6, 4, 5, 7],
//     crc_calc_pad: BitCount(5),
//     crc_polynomial: BitMap(0x2757),   // CRC-14 polynomial without the leading (MSB) 1 0x2757 {8174,8174,18,18,4,4,2,2}
//     crc_width: BitCount(14),
//     crc_start: 0,
//     crc_xor: 0,
//     symbol_bt: 2.0f32,
//     ramp_symbols: SymbolCount(1),
// };

// orig in monitor.rs
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum WindowFunction {
    _Rect,
    _Hann,
    _Hamming,
    _Blackman
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Runtime {
    band_width: Hz,  // **
    channels: RepeatCount, 
    _bit_depth: BitCount,
    rx_symbol_osr: OverSampleMultiplier, // **
    rx_freq_osr: OverSampleMultiplier, // **
    // detector_underload_divisor: RepeatCount, // detector underloads - to increase time res for correlator
    sync_min_score: f32,
    sub_bands: RepeatCount,
    ldpc_max_iteration: RepeatCount,
    auto_segment: bool,
    // subtracts: RepeatCount,
    window_function: WindowFunction,
}

const NYQUIST: f32 = 2.0;

impl Runtime {
    pub const fn rx_freq_osr(&self) -> OverSampleMultiplier {
        self.rx_freq_osr
    }

    pub const fn band_width(&self) -> Hz {
        self.band_width
    }

    pub fn target_input_sample_rate(&self) -> Hz {
        Hz( NYQUIST * self.band_width().0 * self.rx_freq_osr.0 as f32)
    }

    pub const fn bin_res(&self, baud_rate: Hz) -> Hz {
        Hz ( baud_rate.0 / self.rx_freq_osr().0 as f32)
    }

    pub fn input_nfft(&self, baud_rate: Hz) -> RepeatCount {
        RepeatCount((self.target_input_sample_rate().0 / self.bin_res(baud_rate).0 ) as usize)
    }

    pub const fn channels(&self) -> RepeatCount {
        self.channels
    }

    pub fn _target_output_sample_rate(&self) -> Hz {
        self.target_input_sample_rate()
    }

    pub const fn rx_symbol_osr(&self) -> OverSampleMultiplier {
        self.rx_symbol_osr
    }

    pub const fn _bit_depth( &self ) -> BitCount {
        self._bit_depth
    }

    pub const fn sync_min_score(&self) -> f32 {
        self.sync_min_score
    }

    pub const fn sub_bands(&self) -> RepeatCount {
        self.sub_bands
    }

    pub const fn ldpc_max_iteration(&self) -> RepeatCount {
        self.ldpc_max_iteration
    }

    pub const fn auto_segment(&self) -> bool {
        self.auto_segment
    }

    // pub const fn subtracts(&self) -> RepeatCount {
    //     self.subtracts
    // }

    // pub fn detector_underload_divisor(&self) -> RepeatCount {
    //     self.detector_underload_divisor
    // }

    pub fn window_function(&self, i: usize, n: usize) -> f32 {
        // this is obviously not optimised for speed!
        match self.window_function {
            WindowFunction::_Rect => 1.0,
            WindowFunction::_Hann => {
                (std::f32::consts::PI * i as f32 / n as f32).sin().powi(2)
            },
            WindowFunction::_Hamming => {
                let a0 = 0.54;
                let a1 = 0.46;
                let pi2 = 2.0 * std::f32::consts::PI;
                
                a0 - a1 * (pi2 * i as f32 / n as f32).cos()
            },
            WindowFunction::_Blackman => {
                let a0 = 0.42;
                let a1 = 0.5;
                let a2 = 0.08;
                let pi2 = 2.0 * std::f32::consts::PI;
                let x = i as f32 / n as f32;
                
                a0 - a1 * (pi2 * x).cos() + a2 * (2.0 * pi2 * x).cos()
            }
        }
    }
    
}

pub const TEST_FT8_RUNTIME: Runtime = Runtime {
    // should be indep of bandwidth and freq_osr but not there yet
    band_width: Hz(6000.0),  // this is the real design layer - app layer can chose a portion often 250-2500
    channels: RepeatCount(1), 
    _bit_depth: BitCount(32),
    rx_symbol_osr: OverSampleMultiplier(4), // 4
    rx_freq_osr: OverSampleMultiplier(2), // 2
    // detector_underload_divisor: RepeatCount(1), // 2 as per WB2FKO doc
    sync_min_score: 1.0, // 0.4, // 10,
    sub_bands: RepeatCount(1),
    ldpc_max_iteration: RepeatCount(20),
    auto_segment: false, // true, 
    // subtracts: RepeatCount(1),
    window_function: WindowFunction::_Hann,  // Hann in the FT8_lib c code, or Blackman
};
pub const TEST_FREQUENCY: f32 = 1500.0;

pub struct Modem {
    pub protocol: &'static Protocol,
    pub _runtime: &'static Runtime,
    pub _freq_hz: f32,
    pub crc_calc: crc::Crc<u16>,
}

static CRC_ALG: OnceLock<crc::Algorithm<u16>> = std::sync::OnceLock::new();

fn get_crc_alg(protocol:&'static Protocol) -> &'static crc::Algorithm<u16> {
    CRC_ALG.get_or_init(|| {
        // Initialize on first call
        crc::Algorithm {
            width: protocol._crc_width().0 as u8,
            poly: protocol._crc_polynomial().0 as u16,
            init: protocol._crc_start() as u16,
            refin: false,
            refout: false,
            xorout: protocol._crc_xor() as u16,
            check: 0x0,
            residue: 0x0
        }
    })
}

impl Modem {
    pub fn new(protocol:&'static Protocol, runtime: &'static Runtime, freq_hz: f32) -> Modem {
        // This is a recurrent calculation except for the added 0.5
        // what is 0.5 added for - it probably is related to the N/2 + 1 calc
        // let n_spsym = (0.5 + runtime.sample_rate() as f32 * protocol.symbol_period) as usize; // Samples per symbol 
        // let n_wave = protocol.nn() * n_spsym; // Number of output samples

        
        // modem.codeword.resize(protocol.ldpc_n_bytes(), 0);
        // assert_eq!(modem.codeword.len(), modem.protocol.ldpc_n_bytes());

        // modem.l2_tones.resize(protocol.nd(), 0);
        // assert_eq!(modem.l2_tones.len(), modem.protocol.nd());

        // modem.l0_tones.resize(protocol.nn(), 0);
        // assert_eq!(modem.l0_tones.len(), modem.protocol.nn());

        // modem.signal.resize(n_wave, 0f32);
        // assert_eq!(modem.signal.len(), n_wave);

        let crc_alg = get_crc_alg(protocol);
        let crc_calc = crc::Crc::<u16>::new(&crc_alg );

        Modem {
            protocol,
            _runtime: runtime,
            _freq_hz: freq_hz,
            crc_calc
        }
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum XxxError {
    #[error("TODO")]
    _ToDo,
    // #[error("Bad CRC: crc={0}")]
    // _BadCrc(u32),
    #[error("Bad CRC")]
    _BadCrc,
    #[error("Bad ECC")]
    _BadEcc,
    #[error("Bad Msg")]
    _BadMsg,
    #[error("Incomplete Data")]
    _DataIncomplete,
    #[error("Error Message: {0}")]
    _ErrorMessage(String),
    // Configuration(Box<dyn Error + Sync + Send>),
    // InvalidArgument(String),
    // Database(Box<dyn DatabaseError>),
    // Io(Error),
    // Tls(Box<dyn Error + Sync + Send>),
    // Protocol(String),
    #[error("Col not found {0}")]
    _ColNotFound(usize),
    #[error("Row not found {0}")]
    _RowNotFound(usize),
    #[error("Index too high {0}")]
    _IndexTooHigh(usize),
    #[error("Index too low {0}")]
    _IndexTooLow(usize),
    // TypeNotFound {
    //     type_name: String,
    // },
    // ColumnIndexOutOfBounds {
    //     index: usize,
    //     len: usize,
    // },
    // ColumnNotFound(String),
    // ColumnDecode {
    //     index: String,
    //     source: Box<dyn Error + Sync + Send>,
    // },
    // Encode(Box<dyn Error + Sync + Send>),
    // Decode(Box<dyn Error + Sync + Send>),
    // AnyDriverError(Box<dyn Error + Sync + Send>),
    // PoolTimedOut,
    // PoolClosed,
    // WorkerCrashed,
    // Migrate(Box<MigrateError>),
    // InvalidSavePointStatement,
    // BeginFailed,
}


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

#[cfg(test)]
mod tests {
    use super::*;

    pub fn check(protocol: &Protocol) -> bool { 
        (BitCount(protocol.nd().0 * protocol.token_bits().0) == protocol.ldpc_n())
        && (protocol.total_symbols_nn() == SymbolCount(protocol._ns().0 + protocol.nd().0))
        && (SymbolCount(protocol.costas_pattern().len()) == protocol._length_sync())
        && (RepeatCount(protocol.gray_map().len()) == protocol.token_tones())
        && protocol._slot_time_locked()
        && (protocol._total_frame_time().0 <= protocol._slot_time().0)
        // && (self.length_ramp() == 0)
    }

    #[test]
    fn test_constant() {
        assert!(check(&TEST_PROTOCOL));
        // assert!(check(&_JS8A));
        // assert!(check(&_LT8A));
    }
}
