use crate::candidate;
use crate::rustxxx;
// use crate::rustxxx::Secs;
// use crate::rustxxx::TimeStamp;
use crate::waterfall;
// use crate::unpack_ft8;
use crate::layer3;

#[derive(Debug)]
#[derive(Clone)]
pub struct MessageDf {
    pub c_score: f32,
    pub time_secs: rustxxx::Secs,
    pub freq_hz: rustxxx::Hz,
    pub text: String,
}

#[derive(Debug)]
pub struct Message {
    pub df: MessageDf,
    pub codeword: Vec<u8>, // used as key
}

impl Message {
    pub fn new() -> Message {
        Message {
            // used to be a Vec of dfs - not sure why yet
            df: MessageDf{
                c_score: 0.0,
                time_secs: rustxxx::Secs(0.0),
                freq_hz: rustxxx::Hz(0.0),
                text: String::new(), // for debug purposes
            },
            codeword: Vec::new(),
        }
    }
}

fn max2(a: f32, b: f32) -> f32 {
    if a >= b {
        a
    } else {
        b
    }
}

fn max4(a: f32, b: f32, c: f32, d: f32) -> f32 {
    max2(max2(a, b), max2(c, d))
}

pub type DecodeHash = std::collections::HashMap<Vec<u8>, MessageDf>;

#[cfg(any(feature = "enable_rx", test))]
pub struct Decoder {
    protocol: &'static rustxxx::Protocol,
    runtime: &'static rustxxx::Runtime,
}

#[cfg(any(feature = "enable_rx", test))]
impl Decoder {
    pub fn new(
        protocol: &'static rustxxx::Protocol,
        runtime: &'static rustxxx::Runtime, 
    ) -> Decoder {
        Decoder {
            protocol,
            runtime,
        }
    }

    pub fn decode(
        &self, 
        modem: &mut rustxxx::Modem, 
        logls: &Vec<layer3::LogL>,
    ) -> Option<Message> {
        let mut message = Message::new();

        let mut r = modem.ecc_decode_bp(&logls, self.runtime.ldpc_max_iteration().0);
        if r.is_err() {
            r = modem.ecc_decode_bitflip(&logls, self.runtime.ldpc_max_iteration().0);
            if !r.is_err() {
                dbg!("ecc_decode_bitflip contributed");
            }
        }
        match r {
            Ok(codeword) => {
                // dbg!("got past ecc");
                // assert_eq!(modem.protocol, &proto_ft8::FT8);
                // if unpack_ft8::unpack77(&codeword, &mut message.df.text) < 0 {
                //     // dbg!("Bad unpack");
                //     return None;
                // }
                if message.df.text.is_empty() {
                    None
                } else {
                    message.codeword = codeword; // this is messy
                    Some(message)
                }
            }
            Err(_) => {
                // dbg!("Bad ECC");
                None
            }
        }
    }

    pub fn extract_normalised_likelihood(&self, wf: &waterfall::Waterfall, c: &candidate::Candidate) -> Vec<layer3::LogL> {
        let mut logls: Vec<layer3::LogL> = Vec::new(); 

        //Extract 58 bits of symbols - 3 x bits->(syms in logls)
        for bit_idx in 0..self.protocol.nd().0 {
            //Symbol part with Costas array skipped
            let sym_idx = bit_idx + if bit_idx < 29 { 7 } else { 14 };

            //calc block num
            let time_index = c.time_index().0 + (sym_idx * wf.time_osr.0);
            assert!(time_index < wf.time_bins(), "time_index: {}, wf_bins: {}", time_index, wf.time_bins());
            
            let logl = if 
                time_index >= wf.time_bins()
                || c.freq_index().0 >= wf.freq_bins - self.protocol.token_tones().0 * wf.freq_osr.0
            {
                panic!("overrun in decoder"); // layer3::LogL::new(self.protocol)
            } else {
                self.extract_symbol(
                    wf, 
                    rustxxx::TimeIndex(time_index),
                    c.freq_index()
                )
            };
            logls.push(logl);
        }

        //Normalize the llr of each bit
        self.normalize_logl(&mut logls);

        logls
    }

    fn extract_symbol(
        &self, 
        wf: &waterfall::Waterfall, 
        time_index: rustxxx::TimeIndex,
        freq_index: rustxxx::FreqIndex,
    ) -> layer3::LogL {
        let wfl = &wf.line(time_index.0);

        let token_tones = self.protocol.token_tones(); // 8

        let mut s2: Vec<f32> = vec![0.0; token_tones.0 ];

        //Put the tone intensity corresponding to the gray code into s2
        // for j in 0..token_range.0 {
        for (j, s2_item) in s2.iter_mut().enumerate() {
            let x_ofs: usize = self.protocol.gray_map()[j] as usize * self.runtime.rx_freq_osr().0;
            assert!(freq_index.0 + x_ofs < wfl.mag_dbs.len());
            let mag = wfl.mag_dbs[freq_index.0 + x_ofs]; // must be db - as log calcs take place below
            *s2_item = mag as f32;
        }

        let mut logl = layer3::LogL::new(self.protocol);
        //Find the log likelihood ratio (LLR) for each bit for each bit LLR = log(P(b=1)/P(b=0))
        //The LLR of MSB on the gray code is the maximum value of tone 4-7 (1) minus the maximum value of tone 0-3 (0)
        // subtract is divisio as the values are db = log(mag);
        logl.bits[0] = max4(s2[4], s2[5], s2[6], s2[7]) - max4(s2[0], s2[1], s2[2], s2[3]);
        //Similarly, the 2nd bit is the maximum value of tone 2, 3, 6, 7 (1) minus the maximum value of tone 0, 1, 5, 4 (0)
        logl.bits[1] = max4(s2[2], s2[3], s2[6], s2[7]) - max4(s2[0], s2[1], s2[5], s2[4]);
        //Similarly calculate 3bit
        logl.bits[2] = max4(s2[1], s2[3], s2[5], s2[7]) - max4(s2[0], s2[2], s2[6], s2[4]);
        logl
    }

    fn normalize_logl(&self, logls: &mut [layer3::LogL]) {
        assert_eq!(rustxxx::SymbolCount(logls.len()), self.protocol.nd());
 
        let mut sum = 0.0f32;
        let mut sum_of_squares = 0.0f32;

        //Find the normalization coefficient from the variance value of each bit
        for logl in logls.iter() {
            for bit in logl.bits.iter() {
                sum += bit;
                sum_of_squares += bit * bit;    
            }
        }

        let inv_n = 1.0f32 / self.protocol.ldpc_n().0 as f32;
        let variance = (sum_of_squares - (sum * sum * inv_n)) * inv_n;

        //Normalize by multiplying each bit by the normalization factor
        let norm_factor = (24.0f32 / variance).sqrt();
        for logl in logls.iter_mut() {
            for bit in logl.bits.iter_mut() {
                *bit *= norm_factor;
            }
        }
    }

}