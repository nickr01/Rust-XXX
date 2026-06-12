// Layer4 -CRC
// use std::{os::unix::process, result::*};

use crate::rustxxx;

// use crc::{Algorithm, Crc};

// need for Traits
use bitvec::prelude::*;

impl rustxxx::Modem {
    fn _crc_compute(&self, codeword: &[u8]) -> u16 {
        assert!(codeword.len() >= self.protocol._ldpc_p_bytes().0);
        let codeword_bits = codeword.view_bits::<Msb0>();

        let mut payload: Vec<u8> = vec![0; self.protocol._ldpc_p_padded_bytes().0]; 
        // Vec::with_capacity(self.protocol._ldpc_p_padded_bytes().0);
        // payload.resize(self.protocol._ldpc_p_padded_bytes().0, 0);

        let payload_buflen_bits = payload.len() * 8;
        let payload_bits = payload.view_bits_mut::<Msb0>();

        // load just the bits left shifted by the pad
        // https://wsjt.sourceforge.io/FT4_FT8_QEX.pdf  page 8
        // "The CRC is calculated on the source-encoded message, zero-extended from 77 to 82 bits. 
        // Whole bytes for library crc calc input??
        payload_bits[
            payload_buflen_bits-self.protocol._ldpc_p().0-self.protocol._crc_pad_bits().0..payload_buflen_bits-self.protocol._crc_pad_bits().0
        ].copy_from_bitslice(
            &codeword_bits[0..self.protocol._ldpc_p().0]
        );

        self.crc_calc.checksum(&payload)
    }

    #[cfg(any(feature = "enable_tx", test))]
    fn _crc_store(&self, crc_arg: u16, payload: &[u8]) -> Vec<u8> {
        assert_eq!(payload.len(), self.protocol._ldpc_p_bytes().0);
        let mut crc = crc_arg;
        let mut cw_crc = payload.to_owned();
        cw_crc.resize(self.protocol._ldpc_k_bytes().0, 0);
        let codeword_bits = cw_crc.view_bits_mut::<Msb0>();
        for i in self.protocol._ldpc_p().0..self.protocol.ldpc_k().0 {
            codeword_bits.set(i, crc & 0x2000 != 0);
            crc <<= 1;
        }
        cw_crc
    }

    #[cfg(any(feature = "enable_rx", test))]
    pub fn _crc_read(&self, codeword: &[u8]) -> u16 {
        assert_eq!(codeword.len(), self.protocol._ldpc_k_bytes().0);
        let mut crc: u16 = 0;
        let codeword_bits = codeword.view_bits::<Msb0>();
        for i in self.protocol._ldpc_p().0..self.protocol.ldpc_k().0 {
            crc <<= 1;
            if codeword_bits[i] {
                crc |= 1;
            }
        }
        crc
    }

    #[cfg(any(feature = "enable_rx", test))]
    pub fn _crc_check(&self, codeword: &[u8]) -> bool {
        assert_eq!(codeword.len(), self.protocol._ldpc_k_bytes().0);
         let crc1 = self._crc_read(codeword);
        let crc2 = self._crc_compute(codeword);
        crc1 == crc2
    }

    #[cfg(any(feature = "enable_tx", test))]
    fn _crc_add(&self, codeword: &[u8]) -> Vec<u8> {
        let crc = self._crc_compute(codeword);
        self._crc_store(crc, codeword)
    }

    // these are the action stubs
    #[cfg(any(feature = "enable_tx", test))]
    pub fn _l4_crc_add(&self, cw: &[u8]) -> Result<Vec<u8>, rustxxx::XxxError>{
        assert_eq!(cw.len(), self.protocol._ldpc_p_bytes().0);
        Ok(self._crc_add(cw))
    }

    #[cfg(test)]
    pub fn _l4_crc_remove(&self, cw_crc: &Vec<u8>) -> Result<Vec<u8>, rustxxx::XxxError> {
        if self._crc_check(cw_crc) {
            let mut cw = cw_crc.to_owned();
            cw.resize(self.protocol._ldpc_p_bytes().0, 0);
            let resid_bits = (self.protocol._ldpc_k_bytes().0 - self.protocol._ldpc_p_bytes().0) % 8;
            if resid_bits > 0 {
                let codeword_bits = cw.view_bits_mut::<Msb0>();
                for i in self.protocol._ldpc_p().0..self.protocol._ldpc_p().0 + resid_bits {
                    codeword_bits.set(i, false);
                }
            }
            assert_eq!(cw.len(), self.protocol._ldpc_p_bytes().0);
            Ok(cw)
        } else {
            dbg!("bad crc");
            Err(rustxxx::XxxError::_BadCrc)
        }
    }

    #[cfg(test)]
    pub fn l4_outbound(&self, ttl: isize, cw: &Vec<u8>) -> Result<Vec<u8>, rustxxx::XxxError> {
        assert_eq!(cw.len(), self.protocol._ldpc_p_bytes().0);
        let cw_crc = self._l4_crc_add(cw)?;
        assert_eq!(cw_crc.len(), self.protocol._ldpc_k_bytes().0);
        if ttl == 0 {
            self.l4_inbound(&cw_crc)
        } else {
            self.l3_outbound(ttl - 1, &cw_crc)
        }
    }

    #[cfg(test)]
    pub fn l4_inbound(&self, cw_crc: &Vec<u8>) -> Result<Vec<u8>, rustxxx::XxxError> {
        assert_eq!(cw_crc.len(), self.protocol._ldpc_k_bytes().0);
        let cw = self._l4_crc_remove(cw_crc)?;
        assert_eq!(cw.len(), self.protocol._ldpc_p_bytes().0);
        self.l5_top_inbound(&cw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CrcTestData {
        payload: u128,
        checksum: u16,
    }    
    const L5P0: CrcTestData = CrcTestData {
        payload: 0,
        checksum: 0 
    };

    const L5P1: CrcTestData = CrcTestData {
        payload: 0b_11100_00111111_10001010_01101010_11100010_00000111_10100001_11100011_10010100_01010001, // unpadded 77 bits
        // payload: 0b0000011_10000111_11110001_01001101_01011100_01000000_11110100_00111100_01110010_10001010_00100000, // padded
        checksum: 0b_1111_0011_0010, // see crc1.rs
    };

    fn test_roundtrips(modem: &mut rustxxx::Modem, crctestdata: &CrcTestData) {
        // assert_eq!(modem.protocol, &rustxxx::FT8);
        // construct a full message into codeword buffer
        // let mut codeword: [u8; XXX.ldpc_n_bytes()] = [0; XXX.ldpc_n_bytes()];

        let mut codeword: Vec<u8> = Vec::with_capacity(modem.protocol._ldpc_p_bytes().0);
        codeword.resize(modem.protocol._ldpc_p_bytes().0, 0);
        // copy the test binary payload into the codeword buffer 
        {
            let codeword_bits = codeword.view_bits_mut::<Msb0>();
            codeword_bits[0..modem.protocol._ldpc_p().0].store_be::<u128>(crctestdata.payload);
        }

        assert_eq!(codeword.len(), modem.protocol._ldpc_p_bytes().0);

        let crc = {
            let crc = modem._crc_compute(&codeword);
            assert_eq!(crc, crctestdata.checksum);
            crc
        };

        {
            let codeword1 = modem._crc_store(crc, &codeword);
            assert_eq!(modem._crc_read(&codeword1), crc);
            assert_eq!(modem._crc_compute(&codeword1), crc);
            assert!(modem._crc_check(&codeword1));
        }

        // {
        //     assert_eq!(modem._crc_add(), crc);
        //     assert_eq!(modem.crc_read(), crc);
        //     assert_eq!(modem._crc_compute(), crc);
        //     assert!(modem._crc_check());
        // }
    }

    #[test]
    fn test_layer4() {
        let mut modem: rustxxx::Modem = rustxxx::Modem::new(
            &rustxxx::TEST_PROTOCOL, 
            &rustxxx::TEST_FT8_RUNTIME, 
            rustxxx::TEST_FREQUENCY
        );
        test_roundtrips(&mut modem, &L5P0);
        test_roundtrips(&mut modem, &L5P1);
    }
}
