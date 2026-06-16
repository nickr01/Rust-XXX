// Layer3 -ECC
// use std::result::*;

use crate::error;
use crate::types;

use crate::test_generator;
use crate::test_parity;

// use generator::_XXX_LDPC_GENERATOR; // encode
// use parity::{XXX_LDPC_NM, XXX_LDPC_MN}; // decode

use bitvec::prelude::*;

#[derive(Clone)]
pub struct LogL {
    pub bits: Vec<f32>,
}

#[cfg(any(feature = "enable_rx", test))]
impl LogL {
    pub fn new(protocol: &types::Protocol) -> LogL {
        let bits: Vec<f32> = vec![0.0; protocol.token_bits().0];
        LogL {
            bits
        }
    }
}

// Returns 1 if an odd number of bits are set in x, zero otherwise
pub fn _ecc_parity8(mut x: u8) -> u8 {
    x ^= x >> 4; // a b c d ae bf cg dh
    x ^= x >> 2; // a b ac bd cae dbf aecg bfdh
    x ^= x >> 1; // a ab bac acbd bdcae caedbf aecgbfdh
    x % 2 // modulo 2
}

// used in the Sum Product algorithm
// Approximation to speed up tanh/atanh
// #[cfg(feature = "ldpc_bp")]
#[cfg(any(feature = "enable_rx", test))]
fn fast_tanh(x: f32) -> f32 {
    if cfg!(feature = "use_f32tan") {
        x.tanh()
    } else {
        if x < -4.97f32 {
            return -1.0f32;
        }
        if x > 4.97f32 {
            return 1.0f32;
        }
        let x2 = x * x;
        let a = x * (945.0f32 + x2 * (105.0f32 + x2));
        let b = 945.0f32 + x2 * (420.0f32 + x2 * 15.0f32);
        a / b
    }
}

// #[cfg(feature = "ldpc_bp")]
#[cfg(any(feature = "enable_rx", test))]
fn fast_atanh(x: f32) -> f32 {
    if cfg!(feature = "use_f32tan") {
        x.atan()
    } else {
        let x2 = x * x;
        let a = x * (945.0f32 + x2 * (-735.0f32 + x2 * 64.0f32));
        let b = 945.0f32 + x2 * (-1050.0f32 + x2 * 225.0f32);
        a / b
    }
}

impl types::Modem {
        // Encode via LDPC a 91-bit message and return a 174-bit codeword.
    // The generator matrix has dimensions (87,87).
    // The code is a (174,91) regular LDPC code with column weight 3.
    // Arguments:
    // [IN] message   - array of 91 bits stored as 12 bytes (MSB first)
    // [OUT] codeword - array of 174 bits stored as 22 bytes (MSB first)

    // codeword pre-filled with message+CRC and trailing zeros
    #[cfg(any(feature = "enable_tx", test))]
    fn _ecc_encode(&self, 
        // l4_message: &Vec<u8>, // [u8; XXX.ldpc_k_bytes()], 
        // cw out:  mut [u8; XXX.ldpc_n_bytes()]
        cw_crc: &[u8] // mut [u8; XXX.ldpc_k_bytes()]
        // , cw_crc: &Vec<u8>
    ) -> Vec<u8> {
        // assert!(false);
        // This implementation accesses the generator bits straight from the packed binary representation in kXXX_LDPC_generator
        // let mut msg: Vec<u8> = Vec::with_capacity(self.protocol.ldpc_k_bytes());  //  = [0u8; XXX.ldpc_k_bytes()];
        // msg.copy_from_slice(&self.codeword[0..self.protocol.ldpc_k_bytes()]);
        let mut cw_crc_ecc = cw_crc.to_owned();
        cw_crc_ecc.resize(self.protocol().ldpc_n_bytes().0, 0);
        let cw_crc_ecc_bits = cw_crc_ecc.view_bits_mut::<Msb0>();

        // Compute the LDPC checksum bits in the original message and store them back into codeword
        // for i in 0..self.protocol.ldpc_m().0 {
        for (i, row) in test_generator::XXX_LDPC_GENERATOR.iter().enumerate().take(self.protocol().ldpc_m().0) {
            // implementation of bitwise multiplication and parity checking
            // Normally nsum would contain the result of dot product between message and kXXX_LDPC_generator[i],
            // but we only compute the sum modulo 2.
            let mut nsum = 0u8;

            for (j, m) in cw_crc.iter().enumerate().take(self.protocol()._ldpc_k_bytes().0) {
                let bits = m & row[j]; // bitwise AND (bitwise multiplication)
                // let bits = m & _XXX_LDPC_GENERATOR[i][j]; // bitwise AND (bitwise multiplication)
                nsum ^= _ecc_parity8(bits); // bitwise XOR (addition modulo 2)
            }

            if !nsum.is_multiple_of(2) {
                cw_crc_ecc_bits.set(self.protocol().ldpc_k().0 + i, true);
            }
        }
        cw_crc_ecc
    }

    // pub fn xx8_encode(payload: &[u8; XXX.ldpc_k_bytes()], tones: &mut [usize; XXX.nn()]) {
    //     let mut a91 = [0u8; XXX.ldpc_k_bytes()]; // Store 77 bits of payload + 14 bits CRC

    //     // Compute and add CRC at the end of the message
    //     // a91 contains 77 bits of payload + 14 bits of CRC
    //     xxx_add_crc(payload, &mut a91);

    //     let mut codeword = [0u8; XXX.ldpc_n_bytes()];


    //     encode174(&a91, &mut codeword);

    // Check if each bit of Codeword satisfies the check matrix of ldpc
    #[cfg(any(feature = "enable_rx", test))]
    pub fn ecc_check_errors(&self, cw_crc_ecc: &[u8]
        // codeword: &[u8; XXX.ldpc_n_bytes()]
    ) -> usize {
        assert_eq!(cw_crc_ecc.len(), self.protocol().ldpc_n_bytes().0);

        let mut errors: usize = 0;
        let codeword_bits = cw_crc_ecc.view_bits::<Msb0>();

        //Extract one column from check matrix
        for m in test_parity::XXX_LDPC_NM.iter() {
            let mut odd: bool = false;
            for i in m.iter() {
                if *i != 0 {
                    //Take the xor of the bits in the corresponding codeword
                    odd ^= codeword_bits[i - 1]; // if codeword_bits[i - 1] { true } else { false };
                }
            }
            //This line is ok if it is even parity
            if odd {
                errors += 1;
            }
        }
        //If all rows are satisfied (errors = 0), the check matrix is ​​satisfied
        errors
    }

    // pub fn ecc_check_errors(&self, codeword: &Vec<u8>) -> usize {
    //     let mut errors: usize = 0;

    //     let codeword_bits = codeword.view_bits::<Msb0>();

    //     //Extract one column from check matrix
    //     for m in XXX_LDPC_NM {
    //         let mut x: u8 = 0;
    //         for i in m {
    //             if i != 0 {
    //                 //Take the xor of the bits in the corresponding codeward
    //                 x ^= if codeword_bits[i - 1] { 1 } else { 0 };
    //             }
    //         }
    //         //This line is ok if it is even parity
    //         if x != 0 {
    //             errors += 1;
    //         }
    //     }
    //     //If all rows are satisfied (errors = 0), the check matrix is ​​satisfied
    //     errors
    // }

    //
    // Implementation of decoder using product-sum algorithm
    //
    // #[cfg(feature = "ldpc_bp")]
    #[cfg(any(feature = "enable_rx", test))]
    pub fn ecc_decode_bp(&self, 
        logls: &[LogL], // bits
        max_iters: usize,
        // plain_bytes: &mut [u8; XXX.ldpc_n_bytes()],
    ) -> Result<Vec<u8>, error::XxxError> {
        // dbg!("ecc_decode_bp");

        assert_eq!(logls.len(), self.protocol().nd().0);
        let mut plain_bytes: Vec<u8> = vec![0; self.protocol().ldpc_n_bytes().0]; 
        // Vec::with_capacity(self.protocol.ldpc_n_bytes().0);
        // plain_bytes.resize(self.protocol.ldpc_n_bytes().0, 0);

        //Initialize inspection message e
        let mut tov: Vec<[f32; 7]> = Vec::with_capacity(self.protocol().ldpc_n().0);
        tov.resize(self.protocol().ldpc_n().0, [0f32; 7]);

        //Initialize bit message
        let mut toc: Vec<[f32; 7]> = Vec::with_capacity(self.protocol().ldpc_m().0);
        toc.resize(self.protocol().ldpc_m().0,[0.0f32; 7]);

        //Initialize with the maximum value that can have the minimum number of errors
        let mut min_errors = self.protocol().ldpc_m().0;

        //Loop as many times as the product-sum algorithm repeats
        for _it in 0..max_iters {
            let plain = plain_bytes.view_bits_mut::<Msb0>();
            let mut plain_sum: u8 = 0;
            //(1) Test
            let mut n: usize = 0;
            // for n in 0..self.protocol.ldpc_n() {
            for logl in logls.iter() {
                for bit in logl.bits.iter() {
                    //Update each bit of codeword indicated by log likelihood with test message E
                    //(Check messages from 3 check nodes come for 1 bit of codeword)
                    //It is determined by log likelihood Log(P(c=1)/P(c=0)), so '1' if P(c=1)>P(c=0)
                    //If P(c=1)<P(c=0), judge it as '0' and store it in plain[n]
                    let test = (bit + tov[n][0] + tov[n][1] + tov[n][2]) > 0.0f32; // tovs get set in prior iterations
                    plain.set(n, test);
                    n += 1;
                    plain_sum += if test { 1 } else { 0 };
                }
            }

            //If all bits are 0, repeat again
            if plain_sum == 0 {
                break;
            }

            //Check whether the obtained message string satisfies the parity check matrix
            let errors = self.ecc_check_errors(&plain_bytes);
            //Updated minimum number of parity errors
            if errors < min_errors {
                min_errors = errors;
                //Decoding is complete if there are no errors in all bits.
                if errors == 0 {
                    break;
                }
            }
            //(2) Bit message update
            //Update the bit message M from the bit node n connected to each check node m
            //Use log likelihood to determine whether each check node m has a higher probability of 0 or 1 from the perspective of the bit node.
            // for m in 0..self.protocol.ldpc_m().0 {
            for (m, toc_item) in toc.iter_mut().enumerate().take(self.protocol().ldpc_m().0) {
                    //Extract the elements of each row of the check matrix
                for (n_idx, &n) in test_parity::XXX_LDPC_NM[m].iter().enumerate() {
                    if n != 0 {
                        //Find the bit node n connected to the check node
                        let n = n - 1;
                        //The received value of codeword[n] (bit position n) is set as the initial value.
                        let mut tnm = logls[n/self.protocol().token_bits().0].bits[n % self.protocol().token_bits().0];
                        //Add check message e of bit node n (excluding messages coming from node m)
                        // for m_idx in 0..3 {
                        for (m_idx, tov_item) in tov[n].iter().enumerate().take(3) {
                            if (test_parity::XXX_LDPC_MN[n][m_idx] - 1) != m {
                                tnm += tov_item;
                                // tnm += tov[n][m_idx];
                            }
                        }
                        //E = -2 *tanh(-M/2) part of atan(Π tanh(-M/2))
                        toc_item[n_idx] = fast_tanh(-tnm / 2.0f32);
                    }
                }
            }

            //(3) Update of inspection message
            //Update check message E from check node m connected to each bit node n
            //Use log likelihood to determine whether each bit node n has a higher probability of 0 or 1 from the viewpoint of the check node
            // for n in 0..self.protocol.ldpc_n().0 {
            for (n, tov_n) in tov.iter_mut().enumerate().take(self.protocol().ldpc_n().0) {
                for (m_idx, tov_item) in tov_n.iter_mut().enumerate().take(3) {
                    //Find check node m connected to bit node n
                    let m = test_parity::XXX_LDPC_MN[n][m_idx] - 1;
                    let mut tmn = 1.0f32;
                    //Find the product of bit messages m of check node m
                    for (n_idx, &nn) in test_parity::XXX_LDPC_NM[m].iter().enumerate() {
                        if (nn != 0) && (nn - 1) != n {
                            tmn *= toc[m][n_idx];
                        }
                    }
                    // Inspection message E = -2 + atan(Π tanh(-M/2))
                    *tov_item = -2.0f32 * fast_atanh(tmn);
                }
            }
        }
        if min_errors == 0 {
            Ok(plain_bytes)
        } else {
            Err(error::XxxError::_BadEcc)
        }
    }

    //
    //  Implementing a decoder with a bit-flip algorithm
    //
    // #[cfg(feature = "ldpc_bitflip")]
    #[cfg(any(feature = "enable_rx", test))]
    pub fn ecc_decode_bitflip(&self, 
        logls: &[LogL], // bits!!
        max_iters: usize,
        // plain_bytes: &mut [u8; XXX.ldpc_n_bytes()],
    ) -> Result<Vec<u8>, error::XxxError> {
        // dbg!("ecc_decode_bitflip");

        assert_eq!(logls.len(), self.protocol().nd().0);
        let mut plain_bytes: Vec<u8> = vec![0; self.protocol().ldpc_n_bytes().0]; 
        // Vec::with_capacity(self.protocol.ldpc_n_bytes().0);
        // plain_bytes.resize(self.protocol.ldpc_n_bytes().0, 0);

        // Convert soft decision (log (P(x=1) /P(x=0))) to hard decision (0/1)
        {
            let plain = plain_bytes.view_bits_mut::<Msb0>();
            let mut i: usize = 0;
            // plain.copy_from_bitslice(&codeword.map(|x| if x >= 0.0 { true } else { false }));
            for logl in logls.iter() {
                for bit in logl.bits.iter() {
                    plain.set(i, *bit >= 0.0);
                    i += 1;
                }
            }
        }

        for _ in 0..max_iters {
            let plain = plain_bytes.view_bits_mut::<Msb0>();

            //Determine whether each bit in the codeword has more 0 or 1 based on each check node
            let mut votes = vec![vec![0; 2]; self.protocol().ldpc_n().0];

            //Extract elements of check node
            for e in test_parity::XXX_LDPC_NM.iter() {
                //Calculate parity for bit node bi connected from check node
                for bi in e.iter() {
                    if *bi == 0 {
                        continue;
                    }
                    let mut x = 0;
                    //Take xor with bit node other than bit node bi
                    for i in e.iter() {
                        if *i != 0 && *i != *bi {
                            x ^= if plain[*i - 1] { 1 } else { 0 };   // NBNBNBN *i or i?
                        }
                    }
                    //Vote on the ideal value of bit node bi based on the checksum result
                    //If x = 0, node bi votes 0; if x = 1, node bi votes 1.
                    votes[bi - 1][x as usize] += 1;
                }
            }
            // Update each bit of decoding result plain based on voting results
            for i in 0..self.protocol().ldpc_n().0 {
                //If the target bit is 0 and the voting result is 1, it will be flipped to 1.
                if !plain[i] && (votes[i][1] > votes[i][0]) {
                    plain.set(i, true);
                //If the target bit is 1 and the voting result is 0, it will be reversed to 0.
                } else if plain[i] && (votes[i][0] > votes[i][1]) {
                    plain.set(i, false);
                }
            }

            //　Check if check matrix is ​​satisfied
            if self.ecc_check_errors(&plain_bytes) == 0 {
                return Ok(plain_bytes);
            }
        }
        //Error if it does not end after the specified repetition
        Err(error::XxxError::_BadEcc)
    }

    // These are the action stubs
    #[cfg(any(feature = "enable_tx", test))]
    pub fn _l3_ecc_add(&self, cw_crc: &[u8]) -> Result<Vec<u8>, error::XxxError> {
        // TODO encode args not right yet
        Ok(self._ecc_encode(cw_crc))
    }

    #[cfg(test)]
    pub fn _l3_ecc_remove(&self, cw_crc_ecc: &[u8]) ->Result<Vec<u8>, error::XxxError> {
        if self.ecc_check_errors(cw_crc_ecc) == 0 {
            // let codeword_bits = self.codeword.view_bits_mut::<Msb0>();
            // for i in self.protocol.ldpc_k()..self.protocol.ldpc_n() {
            //     // codeword_bits.set(i, false);
            // }
            Ok(cw_crc_ecc[0..self.protocol()._ldpc_k_bytes().0].to_vec())
        } else {
            Err(error::XxxError::_BadEcc)
        }
    }

    #[cfg(test)]
    pub fn l3_outbound(&self, ttl: isize, cw_crc: &Vec<u8>) -> Result<Vec<u8>, error::XxxError>{
        let cw_crc_ecc = self._l3_ecc_add(cw_crc)?;
        if ttl == 0 {
            self.l3_inbound(&cw_crc_ecc)
        } else {
            self.l2_outbound(ttl - 1, &cw_crc_ecc)
        }
    }

    #[cfg(test)]
    pub fn l3_inbound(&self, cw_crc_ecc: &Vec<u8>) ->Result<Vec<u8>, error::XxxError> {
        let cw_crc = self._l3_ecc_remove(cw_crc_ecc)?;
        self.l4_inbound(&cw_crc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    const L4M0: [u8; test_support::TEST_PROTOCOL.ldpc_n_bytes().0] = [ 0xff, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    fn test_roundtrip(modem: &mut types::Modem, l4_message: Vec<u8>) {
        // let mut _l2_codeword= l4_message;  // msg inc crc in place

        let cw_crc_ecc = modem._ecc_encode(
            &l4_message, 
        );
        
        // let mut l2_codeword_bits = [false; XXX.ldpc_n()];
        // unpack_codeword(&l2_codeword, &mut l2_codeword_bits);
        // assert_eq!(ecc_check_errors(&l2_codeword_bits), 0); // check is in bits with no errors
        // assert_eq!(modem.ecc_check_errors(&modem.codeword), 0); // check is in bits with no errors

        let cw_crc_ecc_bits = cw_crc_ecc.view_bits::<Msb0>();
        let mut cw_crc_ecc_bits_f32: Vec<f32> = Vec::with_capacity(modem.protocol().ldpc_n().0);
        cw_crc_ecc_bits_f32.resize(modem.protocol().ldpc_n().0, 0f32);


        let mut cw_crc_ecc_logls: Vec<LogL> = Vec::new();
        cw_crc_ecc_logls.resize(modem.protocol().ldpc_n().0 / modem.protocol().token_bits().0, LogL::new(modem.protocol()));
        assert_eq!(cw_crc_ecc_logls.len(), modem.protocol().nd().0);
        {
            let mut i = 0;
            for ls_idx in 0..modem.protocol().nd().0 {
                for bit in 0..modem.protocol().token_bits().0 {
                    cw_crc_ecc_logls[ls_idx].bits[bit] = if !cw_crc_ecc_bits[i] { -0.01 } else { 1.0 };
                    i += 1;
                }
            }
        }

        {
            // test bp decode
            // let mut l4_message1_bytes = [0u8; XXX.ldpc_n_bytes()];
            let _cw_crc_ecc_bytes = modem.ecc_decode_bp(
                &cw_crc_ecc_logls, 
                20, 
            ).unwrap(); // decode/check are in bits
        }

        {
            // test bitflip decode
            // let mut l4_message1_bytes = [0u8; XXX.ldpc_n_bytes()];
            let _cw_crc_ecc_bytes = modem.ecc_decode_bitflip(
                &cw_crc_ecc_logls, 
                20, 
            ).unwrap(); // decode/check are in bits
        }

        // _________
        
        // let mut l4_message1 = [0u8; XXX.ldpc_n_bytes()];
        // pack_codeword(&l4_message1_bits, &mut l4_message1);

        // let mut l4_message1_k: [u8; _] = [0u8; XXX.ldpc_k_bytes()];  
        // l4_message1_k.copy_from_slice(&l4_message1[0..XXX.ldpc_k_bytes()]);

        // let mut l4_message1_n = [0u8; XXX.ldpc_n_bytes()];
        // pack_bits(&l4_message1_bits, XXX.ldpc_n(), &mut l4_message1_n);

        // assert_eq!(&l4_message1_n, l4_message, "roundtrip failed");
    }
    #[test]
    fn test_layer3() {
        let mut modem: types::Modem = types::Modem::new(
            &test_support::TEST_PROTOCOL, 
            &test_support::TEST_FT8_RUNTIME, 
            test_support::TEST_FREQUENCY
        );

        test_roundtrip(&mut modem, L4M0.to_vec());
    }
}