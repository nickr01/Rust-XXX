use crate::rustxxx;
use crate::debug;
// use crate::rx_streamed;

#[derive(Debug)]
pub struct FreqBinRange {
    from: usize,
    to: usize,
}

impl FreqBinRange {
    pub fn from(&self) -> usize { self.from }
    pub fn to(&self) -> usize { self.to }
}

pub type WflDataType = f32;

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq)]

pub struct WaterfallLine {
    freq_osr: rustxxx::OverSampleMultiplier,
    pub mags: Vec<WflDataType>,
    pub mag_dbs: Vec<WflDataType>,
}

impl WaterfallLine {
    pub fn new(
        freq_bins: usize,
        freq_osr: rustxxx::OverSampleMultiplier,
    ) -> Self {
        let mags: Vec<WflDataType> = vec![0.0; freq_bins];
        let mag_dbs: Vec<WflDataType> = vec![0.0; freq_bins];
        WaterfallLine {
            freq_osr,
            mags,
            mag_dbs,
        }
    }

    // isize args to allow scanning calcs - but will coerce here to usize or fail
    // pub fn index(&self, freq_index: isize, freq_sub: isize) -> usize {
    //     let freq_index: usize = ((freq_index * self.freq_osr.0 as isize) + freq_sub).try_into().expect("frequency index out of range - negative");
    //     if freq_index >= self.wfline.len() {
    //         panic!("frequency index out of range - too high: {freq_index}");
    //     };
    //     freq_index
    // }

    // pub fn read_col(&self, freq_index: isize, freq_sub: isize) -> u8 {
    //     let index = self.index(freq_index, freq_sub);
    //     self.wfline[index]
    // }

    // pub fn write_val(&mut self, freq_index: isize, freq_sub: isize, value: u8) {
    //     let index = self.index(freq_index, freq_sub);
    //     self.wfline[index] = value;
    // }

    pub fn _freq_len(&self) -> usize {
        self.mags.len()
    }

    // pub fn _freq_blocks(&self) -> usize {
    //     self._freq_len()/self.freq_osr.0
    // }
}

type WaterFallLines = Box<circular_buffer::CircularBuffer::<{rustxxx::WATERFALL_BUF_SIZE}, WaterfallLine>>;
pub struct Waterfall {
    pub load_base: u32,
    pub freq_bins: usize,

    pub time_osr: rustxxx::OverSampleMultiplier,   // number of time subdivisions
    pub freq_osr: rustxxx::OverSampleMultiplier,   // number of frequency subdivisions ?>=2

    wflines: WaterFallLines,
    pub magsums: Vec<f32>,
}

impl Waterfall {
    pub fn new(
        freq_bins: usize,
        time_osr: rustxxx::OverSampleMultiplier,
        freq_osr: rustxxx::OverSampleMultiplier,
    ) -> Self {
        let wflines: WaterFallLines = circular_buffer::CircularBuffer::<{rustxxx::WATERFALL_BUF_SIZE}, WaterfallLine>::boxed();
        let magsums: Vec<f32> = vec!(0.0; freq_bins);

        Waterfall {
            load_base: 0,
            freq_bins,

            time_osr,
            freq_osr,

            wflines,
            magsums,
        }
    }

    pub fn line(&self, wfl_num: usize) -> &WaterfallLine {
        &self.wflines[wfl_num]
    }

    pub fn _line_as_mut(&mut self, wfl_num: usize) -> &mut WaterfallLine {
        &mut self.wflines[wfl_num]
    }

    pub fn push_line(&mut self, wfl: WaterfallLine) {
        self.wflines.try_push_back(wfl).unwrap();
        self.load_base += 1;
    }

    pub fn pop_line(&mut self) -> Option<WaterfallLine> {
        self.wflines.pop_front()
    }

    pub fn time_base(&self) -> u32 {
        self.load_base-self.time_bins() as u32
    }

    pub fn time_bins(&self) -> usize {
        self.wflines.len()
    }

    pub fn time_capacity(&self) -> usize {
        self.wflines.capacity()
    }

    pub fn symbols_stored(&self) -> usize {
        self.time_bins()/self.time_osr.0
    }

    pub fn freq_bins(&self) -> usize {
        self.freq_bins
    }

    pub fn _freq_blocks(&self) -> usize {
        self.freq_bins() / self.freq_osr.0
    }

    // pub fn freq_index(&self, freq_index: isize, freq_sub: isize) -> usize {
    //     (freq_index * self.freq_osr.0 as isize + freq_sub) as usize
    // }

    // pub fn time_index(&self, time_block: isize, time_sub: isize) -> usize {
    //     let time_index: usize = (time_block * self.time_osr.0 as isize + time_sub).try_into().expect("time index is negative");
    //     if time_index >= self.time_len()  {
    //         panic!("time index out of range - too high: {time_index}");
    //     }
    //     time_index
    // }

    // pub fn read_row(&self, time_block: isize, time_sub: isize) -> &WaterfallLine {
    //     let time_index = self.time_index(time_block, time_sub);
    //     self.wflines.nth_front(time_index)
    //         .unwrap_or_else(|| panic!("read line from waterfall failed: {time_block}, {time_sub} -> {time_index} into len {}", self.wflines.len()))
    // }

    // this is originally only about determining search ranges and not actual candidates
    pub fn determine_search_freq_bands(&self, num_of_bands: usize, auto_segment: bool ) -> Vec<FreqBinRange> {
        let mut freq_bin_ranges: Vec<FreqBinRange> = Vec::new();
        // if cfg!(feature = "auto_freq_seg") {

        if auto_segment {
            dbg!("auto freq seg");
            // let mut candidate = Candidate {
            //     score: 0,
            //     time_offset: 12,
            //     freq_offset: 0,
            //     time_sub: 0,
            //     freq_sub: 0,
            // };
            let time_magic_12: usize = 12 * self.time_osr.0; // time_offset = 12 why? Maybe to be halfway in slot?

            let mut sum = 0f32;
            for f in 0..self.freq_bins() {
                // candidate.lease= f;
                let m1 = self.wflines[time_magic_12].mags[f]; 
                sum += m1 as f32;
            }

            let th = sum / self.freq_bins() as f32 / 2.0;
            let mut count = 0;
            for f in 0..self.freq_bins() {
                // candidate.freq_offset = f;
                let m1 = self.wflines[time_magic_12].mags[f];
                if m1  > th as WflDataType {  // u8
                    count += 1;
                }
            }

            {
                let average = count / num_of_bands;
                let mut from  = 0;

                let mut count = 0;
                for f in 0..self.freq_bins() {
                    let b4: bool = self.wflines[time_magic_12].mags[f] > th as WflDataType
                        || self.wflines[time_magic_12 + 1].mags[f] > th as WflDataType
                        || self.wflines[time_magic_12 - 1 ].mags[f] > th as WflDataType;
                            
                    if b4 {
                        count += 1;
                    }
                    if count > average {
                        count = 0;
                        let to = f; //  = if f > wf.freq_indep_base_bins { wf.freq_indep_base_bins } else { f };
                        dbg!("Marking band", from, to);
                        freq_bin_ranges.push(FreqBinRange { from, to });
                        from = to;
                    }
                }
            }
            // dbg!("Count of candidate signal triggers for auto range", count);
            freq_bin_ranges
        } else {
            // dbg!("Search ALL freq ranges direct from preset band count", num_of_bands);
            const COSTAS_MAX: usize = 7; 
            let step = self.freq_bins() / num_of_bands;
            for bin in (0..self.freq_bins()).step_by(step) {
                let costas_max = COSTAS_MAX * self.freq_osr.0;
                if bin + step >= self.freq_bins() - costas_max  {
                    // dbg!("last band");
                    freq_bin_ranges.push(FreqBinRange{from: bin, to: self.freq_bins() - costas_max}) 
                } else {
                    // dbg!("intermediate band");
                    freq_bin_ranges.push(FreqBinRange{from: bin, to: bin + step})
                };
            }
            freq_bin_ranges
        }
    }

    // pub fn write_val(&mut self, time_index: usize, time_sub: usize, value: &WaterfallLine) {
    //     self.mag2[freq_sub][freq_index] = value;
    // }

    // pub fn freq_len(&self) -> usize {
    //     self.time_len
    // }


    // fn get_mag(&self, candidate: &Candidate) -> u8 {
    //     assert!(candidate.time_offset >= 0); // it is being used for index generation in this case
    //     let time_offset: usize = candidate.time_offset.try_into().unwrap(); 
    //     let m4 = self.mag4[time_offset][candidate.time_sub].read_val(candidate.freq_offset, candidate.freq_sub);
    //     m4
    // }

    // Dump mag4 spectrogram - should see separate blocks x freq_osr across x axis, and same for y
     pub fn _dump_spectrogram(&self, path: &str) {
        // // can reorder to show interleaving if required
        // for y in 0..self.time_blocks_stored() {
        //     for y_sub in 0..self.time_osr.0 {
        //         let wfl = self.read_row(y, y_sub);
        //         for x in 0..wfl.freq_blocks_stored() {
        //             for x_sub in 0..self.freq_osr.0 {
        //                 let m4 = wfl.read_col(x, x_sub);
        //                 spectr2.push(m4);
        //             }
        //         }
        //     }
        // }

        let mut spectr2 =Vec::new();
        let wflines_iter = self.wflines.iter();
        for wfl in wflines_iter {
            let db_iter = wfl.mag_dbs.iter();
            for db in db_iter {
                spectr2.push(*db);
            }
        }

        debug::_plot_spectrogram_to_file(path ,&spectr2, spectr2.len()/self.wflines.len(), self.wflines.len());
        // plot_spectrogram(path ,&spectr2, self.freq_indep_base_bins * self.freq_osr, wf.mag_time_blocks_num * wf.time_osr);
    }
}

