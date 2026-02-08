use crate::rustxxx;

#[derive(Debug)]
pub struct Candidate {
    time_stamp: rustxxx::TimeStamp,
    time_index: rustxxx::TimeIndex,
    freq_index: rustxxx::FreqIndex,
    score: f32,
}

impl Candidate {
    pub fn new(
    time_stamp: rustxxx::TimeStamp,
        time_index: rustxxx::TimeIndex, 
        freq_index: rustxxx::FreqIndex, 
        score: f32
    ) -> Candidate {
        Candidate {
            time_stamp,
            time_index,
            freq_index,
            score,
        }
    }
    pub fn score(&self) -> f32 {
        self.score
    }
    pub fn time_stamp(&self) -> rustxxx::TimeStamp {
        self.time_stamp
    }
    pub fn time_index(&self) -> rustxxx::TimeIndex {
        self.time_index
    }
    pub fn freq_index(&self) -> rustxxx::FreqIndex {
        self.freq_index
    }
}
