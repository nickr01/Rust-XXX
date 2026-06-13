use crate::types;

#[derive(Debug)]
pub struct Candidate {
    time_base: types::TimeStamp,
    time_index: types::TimeIndex,
    freq_index: types::FreqIndex,
    score: f32,
}

impl Candidate {
    pub fn new(
        time_base: types::TimeStamp,
        time_index: types::TimeIndex, 
        freq_index: types::FreqIndex, 
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
    pub fn time_stamp(&self) -> types::TimeStamp {
        types::TimeStamp(self.time_base.0 + self.time_index.0 as u32)
    }    
    pub fn time_index(&self) -> types::TimeIndex {
        self.time_index
    }
    pub fn freq_index(&self) -> types::FreqIndex {
        self.freq_index
    }
}
