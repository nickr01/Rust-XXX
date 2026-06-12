use std::borrow::Cow;

use crate::rustxxx;

#[derive(Debug)]
#[derive(Clone)]
pub struct Message {
    pub time_secs: rustxxx::Secs,
    pub freq_hz: rustxxx::Hz,
    pub c_score: f32,
    pub codeword: Vec<u8>, // used also as key
    pub delivered: bool,
}

impl Message {
    pub fn new(
        time_secs: rustxxx::Secs,
        freq_hz: rustxxx::Hz,
        c_score: f32,
        codeword: Vec<u8>,
    ) -> Message {
        Message {
            time_secs,
            freq_hz,
            c_score,
            codeword,
            delivered: false,
        }
    }

    pub fn codeword(&self) -> &Vec<u8> {
        &self.codeword
    }

    pub fn key(&self) -> &Vec<u8> {
        self.codeword()
    }

    pub fn is_empty(&self) -> bool {
        self.codeword.is_empty()
    }

    pub fn is_delivered(&self) -> bool {
        self.delivered
    }

    pub fn set_delivered(&mut self) {
        self.delivered = true;
    }

    pub fn is_stale(&self, stale_time: rustxxx::Secs) -> bool {
        // dbg!(self.time_secs.0, stale_time.0);
        return self.time_secs.0 < stale_time.0
    }
}

