// use std::result::*;
// use bitvec::prelude::*;
use crate::rustxxx;

// const _L5_0: [u8; FT8.ldpc_n_bytes()] = [ 0xff, 0xa5, 0x5a, 0x33, 0xfe, 0xff, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

impl rustxxx::Modem {
    #[cfg(any(feature = "enable_tx", test))]
    fn _modulate(&self, cw:&[u8]) -> Result<Vec<f32>, rustxxx::XxxError> {
        let r4 = self._l4_crc_add(cw).expect("Failed to add crc");
        let r3 = self._l3_ecc_add(&r4).expect("Failed to add ecc");
        let r2 = self._l2_gray_encode(&r3).expect("Failed to gray encode");
        let r1 = self._l1_sync_add(&r2).expect("Failed to add sync");
        Ok(self._l0_gfsk_synth(&r1).expect("Failed to encode gfsk"))
    }

    // fn demodulate(&self, signal:&Vec<f32>) -> Result<Vec<u8>, XxxError> {

    // }

    #[cfg(test)]
    pub fn l5_top_outbound(&mut self, ttl: isize, cw:&Vec<u8>) -> Result<Vec<u8>, rustxxx::XxxError> {
        assert_eq!(cw.len(), self.protocol._ldpc_p_bytes().0);
        if ttl == 0 {
            self.l5_top_inbound(cw)
        } else {
            self.l4_outbound(ttl - 1, cw)
        }
    }

    #[cfg(test)]
    pub fn l5_top_inbound(&self, payload:&Vec<u8>) -> Result<Vec<u8>, rustxxx::XxxError> {
        Ok(payload.clone()) // reflector
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::pack_ft8; // ::_pack77;

    // fn test_roundtrip(modem: &mut rustxxx::Modem, s: &str) {
    //     for loopback_at in 0..6 {
    //         let cw = pack_ft8::_pack77(s);
    //         assert_eq!(cw.len(), modem.protocol._ldpc_p_bytes().0);
    //         let loopback_result = modem.l5_top_outbound(loopback_at, &cw).unwrap();
    //         assert_eq!(loopback_result, cw, "failed at loopback {}", loopback_at);
    //     }
    // }

    #[test]
    fn test() {
        
        let mut modem: rustxxx::Modem = rustxxx::Modem::new(
            &rustxxx::TEST_PROTOCOL, 
            &rustxxx::TEST_FT8_RUNTIME, 
            rustxxx::TEST_FREQUENCY
        );

        const M_0: &str = "CQ VK2ZTY QG61";        
        // test_roundtrip(&mut modem, M_0);
    }

}