// layer1 - sync and ramp
// use std::result::*;

#[cfg(any(feature = "enable_tx", test))]
// use clap::error as clap_error;

use crate::error;
use crate::types;

impl types::Modem {
    #[cfg(any(feature = "enable_tx", test))]
    fn _sync_insert(
        &self, l2_tones: &[u8], // [u8; XXX.nd()], 
    ) -> Vec<u8> {
        let mut l0_tones: Vec<u8> = Vec::with_capacity(self.protocol().total_symbols_nn().0);
        // Message structure: S7 D29 S7 D29 S7
        // Total symbols: 79 (XXX.nn())
        assert!(self.protocol()._length_ramp().0 == 0);
        let mut in_tone = 0usize; // Index of the current byte of the codeword
        for i_tone in 0..self.protocol().total_symbols_nn().0 { 
            if i_tone < 7 {
                l0_tones.push(self.protocol().costas_pattern()[i_tone]);
            } else if (36..43).contains(&i_tone) {
                l0_tones.push(self.protocol().costas_pattern()[i_tone - 36]);
            } else if (72..79).contains(&i_tone) {
                l0_tones.push(self.protocol().costas_pattern()[i_tone - 72]);
            } else {
                l0_tones.push(l2_tones[in_tone]);
                in_tone += 1;
            }
        }
        l0_tones
    }

    #[cfg(any(feature = "enable_rx", test))]
    fn _sync_remove(&self, 
        l0_tones: &[u8], //  [u8; XXX.nn()], 
    ) -> Vec<u8> {
        let mut l2_tones: Vec<u8> = Vec::with_capacity(self.protocol().nd().0); //  &mut Vec<u8>, //[u8; XXX.nd()]
        assert!(self.protocol()._length_ramp().0 == 0);
        // let mut out_tone = 0usize; // Index of the current byte of the codeword
        // for i_tone in 0..self.protocol._total_symbols_nn().0 { 
        for (i_tone, l0_tone) in l0_tones.iter().enumerate().take(self.protocol().total_symbols_nn().0) { 
                if i_tone < 7 {
    //            l0_tones[i_tone] = XXX.costas_pattern()[i_tone];
            } else if (36..43).contains(&i_tone) {
                // l0_tones[i_tone] = XXX.costas_pattern()[i_tone - 36];
            } else if (72..79).contains(&i_tone) {
                // l0_tones[i_tone] = XXX.costas_pattern()[i_tone - 72];
            } else {
                // l2_tones.push(l0_tones[i_tone]);
                l2_tones.push(*l0_tone);
            }
        }
        l2_tones
    }

    // These are the action stubs
    #[cfg(any(feature = "enable_tx", test))]
    pub fn _l1_sync_add(&self, 
        l2_tones: &[u8], // [u8; XXX.nd()]
    ) -> Result<Vec<u8>, error::XxxError>{
        // l2_tones -> l0_tones
        // let mut l0_tones: [u8; XXX.nn()] = [0; XXX.nn()];
        // let mut l0_tones: Vec<u8> = Vec::with_capacity(self.protocol.nn());
        Ok(self._sync_insert(l2_tones))
    }

    #[cfg(test)]
    pub fn _l1_sync_remove(&self, 
        l0_tones: &[u8], // [u8; XXX.nn()]
    ) ->Result<Vec<u8>, error::XxxError> {
        // l0_tones -> l2_tones
        // let mut l2_tones: Vec<u8> = Vec::with_capacity(self.protocol.nd()); // [u8; XXX.nd()] = [0; XXX.nd()];
        Ok(self._sync_remove(l0_tones))
    }

    #[cfg(test)]
    pub fn l1_outbound(&self, ttl: isize, 
        l2_tones: &Vec<u8>, freq_hz: types::Hz // [u8; XXX.nd()]
    ) -> Result<Vec<u8>, error::XxxError>{
        // l2_tones -> l0_tones
        // let l0_tones: Vec<u8> = Vec::with_capacity(self.protocol.nn()); // [u8; XXX.nn()] = [0; XXX.nn()];
        let l0_tones = self._l1_sync_add(l2_tones)?;
        if ttl == 0 {
            self.l1_inbound(&l0_tones)
        } else {
            self.l0_outbound(ttl - 1, &l0_tones, freq_hz)
        }
    }

    #[cfg(test)]
    pub fn l1_inbound(&self,
        l0_tones: &Vec<u8>, // [u8; XXX.nn()]
    ) ->Result<Vec<u8>, error::XxxError> {
        // l0_tones -> l2_tones
        // let mut l2_tones: Vec<u8> = Vec::with_capacity(self.protocol.nd()); // [u8; XXX.nd()] = [0; XXX.nd()];
        let l2_tones = self._l1_sync_remove(l0_tones)?;
        self.l2_inbound(&l2_tones)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    fn test_roundtrip(modem: &mut types::Modem, l2_tones: &Vec<u8>) {
        let l0_tones = modem._sync_insert(&l2_tones);
        let l2_tones_c = modem._sync_remove(&l0_tones);
        assert_eq!(*l2_tones, l2_tones_c);
    }

    #[test]
    fn test_layer1() {
        let mut modem: types::Modem = types::Modem::new(
            &test_support::_TEST_PROTOCOL, 
            &test_support::_TEST_FT8_RUNTIME,
        );

        test_roundtrip(&mut modem, &[0u8; test_support::_TEST_PROTOCOL.nd().0].to_vec());
        test_roundtrip(&mut modem, &[5u8; test_support::_TEST_PROTOCOL.nd().0].to_vec());

        let mut msg1 = [0u8; test_support::_TEST_PROTOCOL.nd().0];
        let mut msg2 = [0u8; test_support::_TEST_PROTOCOL.nd().0];
        for i in 0..modem.protocol().nd().0 {
            msg1[i] = (i & 7) as u8;
            msg2[i] = ((i+3) & 7) as u8;
        }
        test_roundtrip(&mut modem, &msg1.to_vec());
        test_roundtrip(&mut modem, &msg2.to_vec());
    }
}