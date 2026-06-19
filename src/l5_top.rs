// use std::result::*;
// use bitvec::prelude::*;
use crate::types;
use crate::error;

// const _L5_0: [u8; FT8.ldpc_n_bytes()] = [ 0xff, 0xa5, 0x5a, 0x33, 0xfe, 0xff, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

impl types::Modem {
    #[cfg(any(feature = "enable_tx", test))]
    pub fn modulate(&self, cw:&[u8], freq_hz: types::Hz) -> Result<Vec<f32>, error::XxxError> {
        let r4 = self._l4_crc_add(cw).expect("Failed to add crc");
        let r3 = self._l3_ecc_add(&r4).expect("Failed to add ecc");
        let r2 = self._l2_gray_encode(&r3).expect("Failed to gray encode");
        let r1 = self._l1_sync_add(&r2).expect("Failed to add sync");
        Ok(self.l0_gfsk_synth(&r1, freq_hz).expect("Failed to encode gfsk"))
    }

    // fn demodulate(&self, signal:&Vec<f32>) -> Result<Vec<u8>, XxxError> {

    // }

    #[cfg(test)]
    pub fn l5_top_outbound(&mut self, ttl: isize, cw:&Vec<u8>, freq_hz: types::Hz) -> Result<Vec<u8>, error::XxxError> {
        assert_eq!(cw.len(), self.protocol()._ldpc_p_bytes().0);
        if ttl == 0 {
            self.l5_top_inbound(cw)
        } else {
            self.l4_outbound(ttl - 1, cw, freq_hz)
        }
    }

    #[cfg(test)]
    pub fn l5_top_inbound(&self, payload:&Vec<u8>) -> Result<Vec<u8>, error::XxxError> {
        Ok(payload.clone()) // reflector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    fn test_roundtrip(modem: &mut types::Modem, cw: Vec<u8>, freq_hz: types::Hz) {
        for loopback_at in 0..6 {
            assert_eq!(cw.len(), modem.protocol()._ldpc_p_bytes().0);
            let loopback_result = modem.l5_top_outbound(loopback_at, &cw, freq_hz).unwrap();
            assert_eq!(loopback_result, cw, "failed at loopback {}", loopback_at);
        }
    }

    #[test]
    fn test() {
        
        let mut modem: types::Modem = types::Modem::new(
            &test_support::TEST_PROTOCOL, 
            &test_support::TEST_FT8_RUNTIME, 
        );

        // const M_0: &str = "CQ VK2ZTY QG61";        

        let cw00: Vec<u8> = vec![0x00u8; modem.protocol()._ldpc_p_bytes().0];
        test_roundtrip(&mut modem, cw00, test_support::TEST_FREQUENCY);

        let cw80: Vec<u8> = vec![0x80u8; modem.protocol()._ldpc_p_bytes().0];
        test_roundtrip(&mut modem, cw80, test_support::TEST_FREQUENCY);

        let cw58: Vec<u8> = vec![0x58u8; modem.protocol()._ldpc_p_bytes().0];
        test_roundtrip(&mut modem, cw58, test_support::TEST_FREQUENCY);

        let cwf8: Vec<u8> = vec![0xf8u8; modem.protocol()._ldpc_p_bytes().0];
        test_roundtrip(&mut modem, cwf8, test_support::TEST_FREQUENCY);
    }

}