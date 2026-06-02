use crate::rustxxx;

#[derive(Debug)]
pub struct Candidate {
    time_base: rustxxx::TimeStamp,
    time_index: rustxxx::TimeIndex,
    freq_index: rustxxx::FreqIndex,
    score: f32,
}

impl Candidate {
    pub fn new(
        time_base: rustxxx::TimeStamp,
        time_index: rustxxx::TimeIndex, 
        freq_index: rustxxx::FreqIndex, 
        score: f32
    ) -> Candidate {
        Candidate {
            time_base,
            time_index,
            freq_index,
            score,
        }
    }
    pub fn score(&self) -> f32 {
        self.score
    }
    pub fn time_stamp(&self) -> rustxxx::TimeStamp {
        rustxxx::TimeStamp(self.time_base.0 + self.time_index.0 as u32)
    }    
    pub fn time_index(&self) -> rustxxx::TimeIndex {
        self.time_index
    }
    pub fn freq_index(&self) -> rustxxx::FreqIndex {
        self.freq_index
    }
}
