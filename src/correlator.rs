use crate::rustxxx;
use crate::debug;
use crate::candidate;
use crate::waterfall;

pub struct Correlator {
    protocol: &'static rustxxx::Protocol,
    runtime: &'static rustxxx::Runtime,
}

impl Correlator {
    pub fn new(
        protocol: &'static rustxxx::Protocol,
        runtime: &'static rustxxx::Runtime, 
    ) -> Correlator {
        Correlator {
            protocol,
            runtime,
        }
    }

    pub fn find_freq_candidates(
        &self,
        wf: &waterfall::Waterfall, 
        freq_bin_range: &waterfall::FreqBinRange,
    ) -> Option<Vec<candidate::Candidate>> {
        // dbg!("entry");

        let mut candidates: Vec<candidate::Candidate> = Vec::new();
        assert!(wf.symbols_stored() >= self.protocol.nd().0);
 
        for freq_index in 
                freq_bin_range.from() + wf.freq_osr.0 
                .. freq_bin_range.to() - (self.protocol.token_tones().0 * wf.freq_osr.0)
        {
            let score = self.score_sync_correlation(
                wf, 
                rustxxx::FreqIndex(freq_index),
            );
            if score < self.runtime.sync_min_score()  {
                continue;
            }
            // dbg!(score);
            candidates.push(
                candidate::Candidate::new(
                    rustxxx::TimeStamp(wf.time_base()),
                    rustxxx::TimeIndex(0),
                    rustxxx::FreqIndex(freq_index),
                    score, 
                )
            );
        }

        // candidates.sort_by_key(|b| std::cmp::Reverse(b.freq_index().0));
        candidates.sort_by_key(|b| b.freq_index().0);

        // self.dump_histogram(wf, &candidates);
        Some(candidates)
    }

    //Calculate the score based on the correlation between the signal of the target candidate and the Costas array
    pub fn score_sync_correlation(
        &self, 
        wf: &waterfall::Waterfall, 
        freq_index_base: rustxxx::FreqIndex
    ) -> f32 {
        if wf.symbols_stored() < self.protocol.nd().0 {
            return 0.0;
        };

        // dbg!(time_index_base);
        // dbg!(freq_index_base);

        // assert!(time_index_base.0 >= wf.time_osr.0);
        assert!(freq_index_base.0 >= wf.freq_osr.0);
        
        let mut sync_sum: f32 = 0.0;
        let mut norm_sum: f32 = 0.0;

        let scan_max = self.protocol.costas_pattern().len();

        //Loop through the potential 3 Costas array locations
        for sync_block_num in 0..self.protocol.num_sync().0 {
            let sync_block_index = sync_block_num * self.protocol.sync_offset().0; // -> 0, 36, 72
            // dbg!(sync_block_index);
            //Loop over each element of Costas array
            for (costas_index, costas_sym) in self.protocol.costas_pattern().iter().enumerate() {
                //The starting position of the Costas array is bits 0, 36, and 72.
                let sync_symbol_index = (sync_block_index + costas_index) * wf.time_osr.0;

                let time_index = rustxxx::TimeIndex(sync_symbol_index); 
                if time_index.0 > wf.time_bins() { // } - wf.time_osr.0 {
                    dbg!("run out of lines", time_index.0);
                    continue;
                }

                if freq_index_base.0 >= wf.freq_bins() - (scan_max * self.runtime.rx_freq_osr().0) {
                    continue;
                }

                // at the moment is only a freq axis test whereas time & freq did better before
                for scan_tone in 0..scan_max {
                    let freq_index = freq_index_base.0 + (scan_tone as usize * self.runtime.rx_freq_osr().0);
                    let mag = wf.line(time_index.0).mags[freq_index];
                    if scan_tone as usize == *costas_sym as usize {
                        sync_sum += mag;
                    } else {
                        norm_sum += mag;
                    }
                }
            }
        }

        // norm_sum = norm_sum - sync_sum / 8 as f32; // normalise
        norm_sum = norm_sum - sync_sum / (scan_max - 1) as f32; // normalise

        if norm_sum > 0.0 {
            sync_sum/norm_sum
        } else {
            0.0
        }
    }

    fn _dump_histogram(&self, wf: &waterfall::Waterfall, candidates: &[candidate::Candidate]) {
        let max_bin = wf.freq_bins();
        let mut counts = vec![0.0f32; max_bin];
        let mut scores = vec![0.0f32; max_bin];
        for candidate in candidates.iter().take(max_bin) {
            counts[candidate.freq_index().0] += 1.0;
            scores[candidate.freq_index().0] += candidate.score() as f32;
            // dbg!(candidate);
        }

        debug::_plot_graph(
            "out/magbins.png", 
            "Magsum",
            &wf.magsums,
            0, wf.magsums.len(),
            0.0, wf.magsums.clone().into_iter().reduce(f32::max).unwrap()
        );

        debug::_plot_graph(
            "out/correl_counts.png", 
            "Correlator counts",
            &counts,
            0, counts.len(),
            0.0, counts.clone().into_iter().reduce(f32::max).unwrap()
        );

        debug::_plot_graph(
            "out/correl_scores.png", 
            "Correlator scores",
            &scores,
            0, scores.len(),
            0.0, scores.clone().into_iter().reduce(f32::max).unwrap()
        );

    }
}