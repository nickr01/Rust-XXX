//Layer2 bit level <-> Gray Code
// use std::result::*;

use crate::error;
use crate::types;

// use generator::XXX_LDPC_GENERATOR;
// use crc::*;

impl types::Modem {
    // pub fn xx8_encode(payload: &[u8; XXX.ldpc_k_bytes()], tones: &mut [usize; XXX.nn()]) {
    //     let mut a91 = [0u8; XXX.ldpc_k_bytes()]; //Store 77 bits of payload + 14 bits CRC

    //     // Compute and add CRC at the end of the message
    //     // a91 contains 77 bits of payload + 14 bits of CRC
    //     xxx_add_crc(payload, &mut a91);

    //     let mut self.codeword = [0u8; XXX.ldpc_n_bytes()];

    //     encode174(&a91, &mut self.codeword);

    // gray encode codewords into tones
    #[cfg(any(feature = "enable_tx", test))]
    fn gray_encode(
        &self,
        codeword: &[u8], // [u8; XXX.ldpc_n_bytes()],
                         // self.l2_tones: &mut [u8; XXX.nd()]
    ) -> Vec<u8> {
        let mut l2_tones: Vec<u8> = Vec::with_capacity(self.protocol().nd().0);

        let mut mask = 0x80u8; // Mask to extract 1 bit from self.codeword
        let mut i_byte = 0usize; // Index of the current byte of the self.codeword

        for _i_tone in 0..self.protocol().nd().0 {
            // Extract 3 bits from self.codeword at i-th position
            let mut bits3 = 0u8;

            if (codeword[i_byte] & mask) != 0 {
                bits3 |= 4;
            }

            mask >>= 1;
            if mask == 0 {
                mask = 0x80u8;
                i_byte += 1;
            }

            if (codeword[i_byte] & mask) != 0 {
                bits3 |= 2;
            }

            mask >>= 1;
            if mask == 0 {
                mask = 0x80u8;
                i_byte += 1;
            }

            if (codeword[i_byte] & mask) != 0 {
                bits3 |= 1;
            }

            mask >>= 1;
            if mask == 0 {
                mask = 0x80u8;
                i_byte += 1;
            }
            //Convert 3bit to 8 tones
            //Since it is a Gray code, the Hamming distance (the number of bits that differ) from the bit pattern of the adjacent tone is 1.
            //Even if the frequency changes due to Doppler etc., there is a high possibility that it can be corrected by error correction.
            l2_tones.push(self.protocol().gray_map()[bits3 as usize]);
        }
        l2_tones
    }

    // gray decode tones into codewords
    #[cfg(any(feature = "enable_rx", test))]
    pub fn _gray_decode(
        &self,
        l2_tones: &[u8], //  [u8; XXX.nd()],
    ) -> Vec<u8> {
        // Message structure: S7 D29 S7 D29 S7
        // Total symbols: 79 (XXX.nn())
        let mut codeword: Vec<u8> = vec![0; self.protocol().ldpc_n_bytes().0]; // Vec::with_capacity(self.protocol.ldpc_n_bytes().0); // )&mut [u8; XXX.ldpc_n_bytes()],
                                                                               // codeword.resize(self.protocol.ldpc_n_bytes().0, 0);

        let mut mask = 0x80u8; // Mask to set 1 bit into self.codeword
        let mut i_byte = 0usize; // Index of the current byte of the self.codeword

        // for i_tone in 0..self.protocol.nd().0 {
        for l2_tone in l2_tones.iter().take(self.protocol().nd().0) {
            // Convert 8 tones to 3bit
            // Extract the 3 bits into self.codeword at i-th position

            // let bits3 = self.protocol._gray_rmap()[l2_tones[i_tone] as usize];
            let bits3 = self.protocol()._gray_rmap()[*l2_tone as usize];

            // dbg!(bits3);

            if (bits3 & 4) != 0 {
                codeword[i_byte] |= mask;
            }
            mask >>= 1;
            if mask == 0 {
                mask = 0x80u8;
                i_byte += 1;
            }

            if (bits3 & 2) != 0 {
                codeword[i_byte] |= mask;
            }
            mask >>= 1;
            if mask == 0 {
                mask = 0x80u8;
                i_byte += 1;
            }

            if (bits3 & 1) != 0 {
                codeword[i_byte] |= mask;
            }
            mask >>= 1;
            if mask == 0 {
                mask = 0x80u8;
                i_byte += 1;
            }
        }
        codeword
    }

    // these are the action stubs
    #[cfg(any(feature = "enable_tx", test))]
    pub fn _l2_gray_encode(
        &self,
        codeword: &[u8], // mut [u8; XXX.ldpc_n_bytes()]
    ) -> Result<Vec<u8>, error::XxxError> {
        // let mut tones = [0u8; XXX.nd()];
        // codeword -> l2_tones
        Ok(self.gray_encode(codeword))
    }

    #[cfg(test)]
    pub fn _l2_gray_decode(
        &self,
        l2_tones: &[u8], // [u8; XXX.nd()]
    ) -> Result<Vec<u8>, error::XxxError> {
        // l2_tones -> codeword
        // let mut codeword = [0u8; XXX.ldpc_n_bytes()];
        Ok(self._gray_decode(l2_tones))
    }

    #[cfg(test)]
    pub fn l2_outbound(
        &self,
        ttl: isize,
        codeword: &Vec<u8>,
        freq_hz: types::Hz,
    ) -> Result<Vec<u8>, error::XxxError> {
        // let mut tones = [0u8; XXX.nd()];
        // codeword -> l2_tones
        let l2_tones = self._l2_gray_encode(codeword)?;
        if ttl == 0 {
            self.l2_inbound(&l2_tones)
        } else {
            self.l1_outbound(ttl - 1, &l2_tones, freq_hz)
        }
    }

    #[cfg(test)]
    pub fn l2_inbound(
        &self,
        l2_tones: &Vec<u8>, // [u8; XXX.nd()]
    ) -> Result<Vec<u8>, error::XxxError> {
        // l2_tones -> codeword
        // let mut codeword = [0u8; XXX.ldpc_n_bytes()];
        let cw_crc_ecc = self._l2_gray_decode(l2_tones)?;
        self.l3_inbound(&cw_crc_ecc)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    fn recheck_config(modem: &types::Modem) {
        assert_eq!(
            modem.protocol().nd().0 * modem.protocol().token_bits().0,
            modem.protocol().ldpc_n().0
        );
    }

    fn test_roundtrip(modem: &types::Modem, codeword_in: &Vec<u8>) {
        assert_eq!(codeword_in.len(), modem.protocol().ldpc_n_bytes().0);
        // modem.codeword = codeword_in.clone();

        // let mut tones = [0u8; XXX.nd()];
        let tones = modem.gray_encode(codeword_in);

        // let mut codeword_out = [0u8; XXX.ldpc_n_bytes()];
        let codeword_out = modem._gray_decode(&tones);
        assert_eq!(codeword_out, *codeword_in);
    }

    const MSG0: [u8; test_support::_TEST_PROTOCOL.ldpc_n_bytes().0] = [
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 0xfc,
    ];
    const MSG1: [u8; test_support::_TEST_PROTOCOL.ldpc_n_bytes().0] = [
        67, 171, 17, 12, 2, 2, 76, 47, 161, 170, 70, 55, 40, 30, 2, 1, 0, 251, 55, 25, 213, 0xfc,
    ];

    #[test]
    fn test_layer2() {
        let mut modem: types::Modem = types::Modem::new(
            &test_support::_TEST_PROTOCOL,
            &test_support::_TEST_FT8_RUNTIME,
        );
        recheck_config(&modem);
        test_roundtrip(&mut modem, &MSG0.to_vec());
        test_roundtrip(&mut modem, &MSG1.to_vec());
    }
}
