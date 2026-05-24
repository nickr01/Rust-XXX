
use crate::candidate;
use crate::detector;
use crate::correlator;
use crate::decoder;
use crate::detector::DetectFFT;
use crate::rustxxx;
use crate::waterfall;
// use crate::waterfall::Waterfall;

// use realfft::RealFftPlanner;

// use std::cell::UnsafeCell;
// use std::collections::HashMap;
// use std::thread::yield_now;
// use std::time::Instant;

pub struct Receiver {
    pub start_time: std::time::Instant,
    pub nfft: usize, 
    pub protocol: &'static rustxxx::Protocol,
    pub runtime: &'static rustxxx::Runtime,
}

impl Receiver {

    pub fn new(
        protocol: &'static rustxxx::Protocol,
        runtime: &'static rustxxx::Runtime, 
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

    fn get_df(&self, c: &candidate::Candidate, wf: &waterfall::Waterfall) -> (rustxxx::Hz, rustxxx::Secs) {
        let freq_hz: rustxxx::Hz =
            rustxxx::Hz(((c.freq_index().0 as f32 + 1.0)/ wf.freq_osr.0 as f32) / self.protocol.symbol_period().0);
        let time_secs: rustxxx::Secs =
            rustxxx::Secs((c.time_stamp().0 as f32 / wf.time_osr.0 as f32) * self.protocol.symbol_period().0);
        (freq_hz, time_secs)
    }

    fn try_waterfall_decode(
        &self,
        detector: &mut detector::Detector,
        correlator: &mut correlator::Correlator,
        message_hash: &mut decoder::DecodeHash
    ) -> usize{
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

                        let mut modem: rustxxx::Modem = rustxxx::Modem::new(
                            self.protocol, 
                            self.runtime, 
                            rustxxx::TEST_FREQUENCY
                        );

                        let decoder = decoder::Decoder::new(self.protocol, self.runtime);

                        let mut success = 0;

                        for c in candidates.iter_mut() {
                            let logls = decoder.extract_normalised_likelihood(&detector.wf, c);
                            match decoder.decode(&mut modem, &logls) {
                                Some(mut message) => {
                                    let (freq_hz, time_secs)  = self.get_df(c, &detector.wf);
                                    message.df = decoder::MessageDf{ 
                                        c_score: c.score(), time_secs, freq_hz, text: message.df.text
                                    };

                                    // let message_key = modem.crc_read(message);
                                    // now using message.txt as the key, and message.df as stored value

                                    let should_store = match message_hash.get(&message.codeword) {
                                        None => {
                                            true
                                        },
                                        Some(stored_msg) => {
                                            false
                                            // if stored_msg.c_score < message.df.c_score {
                                            //     // dbg!("Upgraded message");
                                            //     true
                                            // } else {
                                            //     false
                                            // }
                                        }
                                    };
                                    if should_store {
                                        dbg!(&message.df.text);
                                        message_hash.insert(message.codeword.clone(), message.df);
                                        success += 1;
                                    }
                                    pass_decodes += success;
                                    // dbg!(
                                    //     // pass,
                                    //     freq_bin_range.from(),
                                    //     freq_bin_range.to(),
                                    //     success,
                                    //     // candidates.len()
                                    // );
                                },
                                None => {}
                            }
                        };
                    } else {
                        // dbg!("No candidates");
                    }
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
        if detector.wf.symbols_stored() >= self.protocol.total_symbols_nn().0 + 1 {
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
    ) -> u32 {
        let sample_count = 1u32;

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
            }     
            buf.push(sample * detector.window_function_samples[buf.len()]); // NB window happens here
        }
        sample_count
    }


}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test() {
    }

}