use crate::candidate;
use crate::correlator;
use crate::decoder;
use crate::detector;
use crate::detector::DetectFFT;
use crate::message;
use crate::types;
use crate::waterfall;

// use crate::waterfall::Waterfall;

// use realfft::RealFftPlanner;

// use std::cell::UnsafeCell;
// use std::collections::HashMap;
// use std::thread::yield_now;
// use std::time::Instant;

#[cfg(any(feature = "enable_rx", test))]
pub struct Receiver {
    pub start_time: std::time::Instant,
    pub nfft: usize, 
    pub protocol: &'static types::Protocol,
    pub runtime: &'static types::Runtime,
}

#[cfg(any(feature = "enable_rx", test))]
impl Receiver {

    pub fn new(
        protocol: &'static types::Protocol,
        runtime: &'static types::Runtime, 
    ) -> Receiver {
        dbg!(runtime.band_width());
        
        dbg!(protocol.symbol_period());

        let baud_rate = protocol.baud_rate();
        dbg!(baud_rate);

        let nfft = runtime.input_nfft(baud_rate);
        dbg!(nfft);

        let bins = (nfft.0/2) + 1;
        dbg!(bins);

        let base_bins = bins/runtime.rx_freq_osr().0;
        dbg!(base_bins);

        let res = runtime.target_input_sample_rate().0 / nfft.0 as f32;
        dbg!(res);
        assert_eq!(res, baud_rate.0/runtime.rx_freq_osr().0 as f32);

        // single instance caches plans
        
        Receiver {
            start_time: std::time::Instant::now(),
            nfft: nfft.0,
            protocol,
            runtime,
        }
    }

    fn try_waterfall_decode(
        &self,
        detector: &mut detector::Detector,
        correlator: &mut correlator::Correlator,
        message_hash: &mut decoder::DecodeHash
    ) -> usize {
        // dbg!("entry");
        // TODO: disable or remove this
        assert!(!self.runtime.auto_segment()); // blocking auto for the moment
        let freq_bin_ranges: Vec<waterfall::FreqBinRange> = detector.wf.determine_search_freq_bands(
            self.runtime.sub_bands().0, self.runtime.auto_segment()
        );

        let mut pass_decodes: usize = 0;
        // dbg!(self.runtime.sub_bands().0, &freq_bin_ranges);

        for freq_bin_range in freq_bin_ranges {
            match correlator.find_freq_candidates(&detector.wf, &freq_bin_range) {
                Some(mut candidates) => {
                    if candidates.len() > 0 {
                        // dbg!(candidates.len());
                        // dbg!(&candidates);

                        let mut modem: types::Modem = types::Modem::new(
                            self.protocol, 
                            self.runtime, 
                            None
                        );

                        let decoder = decoder::Decoder::new(self.protocol, self.runtime);

                        let mut success = 0;

                        for c in candidates.iter_mut() {
                            let logls = decoder.extract_normalised_likelihood(&detector.wf, c);

                            let time_secs =
                                types::Secs(
                                   ((c.time_stamp().0 + c.time_index().0 as u32) as f32 / detector.wf.time_osr.0 as f32) * self.protocol.symbol_period().0
                                );

                            let freq_hz =
                                types::Hz(
                                    ((c.freq_index().0 as f32 + 1.0)/ detector.wf.freq_osr.0 as f32) / self.protocol.symbol_period().0
                                );

                            match decoder.decode(time_secs, freq_hz, c.score(), &mut modem, &logls) {
                                Some(message) => {
                                    if !message.is_empty() {
                                        if !message_hash.contains_key(message.key()) {
                                            message_hash.insert(message.key().clone(), message);
                                            success += 1;
                                        }
                                        pass_decodes += success;
                                    }
                                },
                                None => {}
                            }
                        };
                    };
                }
                None => {},
            }
        }
        pass_decodes
    }

    fn proc_nfft_buffer(
        &self, 
        rfft_nfft_f: &DetectFFT,
        fft_input_vec: &mut Vec<f32>,
        detector: &mut detector::Detector,
        correlator: &mut correlator::Correlator,
        message_hash: &mut decoder::DecodeHash,
    ) -> usize {
        // dbg!(fft_input_vec.len());
        let mut pass_decodes = 0;
        assert_eq!(fft_input_vec.len(), self.nfft); 
        detector.add_wfline(fft_input_vec, rfft_nfft_f);
        // dbg!(detector.wf.time_blocks());
        if detector.wf.symbols_stored() >= self.protocol.total_symbols_nn().0 + detector.wf.symbol_pad() {
            pass_decodes = self.try_waterfall_decode(
                detector,
                correlator,
                message_hash,
            );
            let _wfl = &detector.wf.pop_line();
            // TODO: push _wfl into subtractor queue
            if pass_decodes > 0 
            {
                // dbg!(pass_decodes);
            };
        }
        pass_decodes
    }

    // consume sample into waterfall
    pub fn load_sample_into_waterfall_lines(&mut self,
        sample: f32,
        rfft_nfft_f: &DetectFFT,
        detector_input_bufs: &mut detector::DetectorInputBuffs,
        detector: &mut detector::Detector,
        correlator: &mut correlator::Correlator,
        message_hash: &mut decoder::DecodeHash,
    ) -> usize {
        let mut bufs_consumed: usize = 0;

        for buf in detector_input_bufs.iter_mut()
        {
            if buf.len() == self.nfft {
                self.proc_nfft_buffer(
                    rfft_nfft_f,
                    buf,
                    detector,
                    correlator,
                    message_hash,
                );
                buf.clear();
                assert_eq!(buf.len(), 0);
                bufs_consumed += 1;
            }     
            buf.push(sample * detector.window_function_samples[buf.len()]); // application of window happens here
        }
        bufs_consumed
    }


}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test() {
    }

}