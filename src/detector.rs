use crate::types;
use crate::types::RepeatCount;
use crate::waterfall;

use realfft::RealToComplex;
// use ringbuf::traits::Consumer;
use std::sync::Arc;

pub type DetectFFT = std::sync::Arc<dyn realfft::RealToComplex<f32>>;

#[cfg(any(feature = "enable_rx", test))]
pub struct Detector {
    // runtime: constant::Runtime,
    // protocol: constant::Protocol,
    nfft: RepeatCount, // FFT size
    // underload_divisor: constant::RepeatCount,
    pub _min_detected_mag: f32, // (debug stats)
    pub _max_detected_mag: f32, // (debug stats)

    // pub wf_under_load: waterfall::Waterfall,
    pub wf: waterfall::Waterfall,
    pub window_function_samples: Vec<f32>,
}

#[cfg(any(feature = "enable_rx", test))]
fn build_window_function_samples(nfft: usize, runtime: &types::Runtime) -> Vec<f32> {
    let mut window_function_samples = Vec::with_capacity(nfft);
    let fft_norm = 2.0f32 / nfft as f32;
    for i in 0..nfft {
        window_function_samples.push(fft_norm * runtime.window_function(i, nfft));
    }
    window_function_samples.resize(nfft, 0.0);
    window_function_samples
}

pub type DetectorInputBuffs = Vec<Vec<f32>>;

#[cfg(any(feature = "enable_rx", test))]
impl Detector {
    pub fn new(
        runtime: types::Runtime,
        protocol: types::Protocol,

        // real_fft: &mut realfft::RealFftPlanner<f32>,
        nfft: RepeatCount,
    ) -> Self {
        let window_function_samples = build_window_function_samples(nfft.0, &runtime);

        let wf = waterfall::Waterfall::new(
            protocol.total_symbols_nn().0 * runtime.rx_symbol_osr().0,
            nfft.0 / (4 * runtime.rx_freq_osr().0), // the sample rate is up because of osr but useful bin proportion is less
            runtime.rx_symbol_osr(),
            runtime.rx_freq_osr(),
        );

        Detector {
            // runtime,
            // protocol,
            nfft,
            // rfft_nfft_f,
            _min_detected_mag: f32::MAX,
            _max_detected_mag: f32::MIN,
            wf,
            window_function_samples,
        }
    }

    pub fn add_wfline(
        &mut self,
        fft_input_vec: &mut [f32],
        rfft_nfft_f: &Arc<dyn RealToComplex<f32>>,
    ) {
        let wfl = self.proc_time_sub(fft_input_vec, rfft_nfft_f);
        self.wf.push_line(wfl);
    }

    // pub fn wload_input_vec(
    //     &mut self, samples: &mut rustxxx::InputBufReader
    // ) -> Option<Vec<f32>>  {
    //     //Find the beginning and end of the sample sequence subject to STFT
    //     //Here, subblock_size is the size of the symbol period divided by the oversample in the time direction
    //     let load_count = self.nfft; //  / underload_divisor.0;

    //     // if samples.len() < load_count {
    //     //     dbg!("incomplete time sub_block - process deferred");
    //     //     return None;
    //     // }

    //     // load a block, len = load_count, from the input buffer
    //     // window, then resize to nfft
    //     let mut fft_input_vec = Vec::with_capacity(self.nfft);

    //     let mut samples_iter = samples.pop_iter();
    //     {
    //         let mut i = 0;
    //         while i < load_count {
    //             // load and window
    //             // NB not actually consuming samples - ACTUALLY WE ARE NOW!!!
    //             match samples_iter.next() {
    //                 Some(sample) => {
    //                     fft_input_vec.push(sample * self.window_function_samples[i]);
    //                     i += 1;
    //                 },
    //                 None => {
    //                     unreachable!("Consumed samples early");
    //                 }
    //             }
    //         }
    //     }
    //     fft_input_vec.resize(self.nfft, 0.0); // 0 pad if underloaded
    //     Some(fft_input_vec)
    // }

    pub fn proc_time_sub(
        &self,
        fft_input_vec: &mut [f32],
        rfft_nfft_f: &Arc<dyn RealToComplex<f32>>, // FFT forward
    ) -> waterfall::WaterfallLine {
        assert_eq!(fft_input_vec.len(), self.nfft.0);

        let mut fft_output_vec = rfft_nfft_f.make_output_vec();
        assert_eq!(fft_output_vec.len(), self.nfft.0 / 2 + 1);

        // Execute real fft
        rfft_nfft_f
            .process(fft_input_vec, &mut fft_output_vec)
            .unwrap();

        // The FFT result is obtained as a complex number in the output. Size is nfft /2 + 1 excluding aliases
        // Here the frequency of each bin[n] f(n) = (fs /nfft) *n = (12000 /3480) *n = 3.125 *n (Hz)
        // Obtain the power spectrum in units of oversamples in the frequency direction

        // assert_eq!(self.wf.freq_indep_base_bins * self.wf.freq_osr.0, fft_output_vec.len());

        assert_eq!(self.nfft.0 / 2 + 1, fft_output_vec.len());
        let mut wfl = waterfall::WaterfallLine::new(self.wf.freq_bins(), self.wf.freq_osr);

        // let mut tmp_max: f32 = f32::MIN;

        let mut bin: usize = 0;
        // skip DC bin
        for fft_output_bin in fft_output_vec.iter().skip(1) {
            //Calc the power of the bin, scale and distribute into the mag4 array
            // let mag2 = fft_output_vec[fft_bin_idx].norm_sqr();
            let mag2 = fft_output_bin.norm_sqr();
            // if mag2 > tmp_max {
            //     tmp_max = mag2;
            // }

            //Convert to decibel and scale to 8bit
            let db = 10.0 * (1e-12 + mag2).log10();

            // // debug only
            // if db > self.max_detected_mag {
            //     self.max_detected_mag = db;
            // }
            // if db < self.min_detected_mag {
            //     self.min_detected_mag = db
            // }

            {
                // let scaled = 2.0 * db + 3.0 * (<waterfall::WflDataType>::BITS as f32) * 10.0;
                let scaled = 2.0 * db + 3.0 * 8.0 * 10.0;

                let mag_db: waterfall::WflDataType = if scaled > <waterfall::WflDataType>::MAX {
                    <waterfall::WflDataType>::MAX
                } else {
                    scaled as waterfall::WflDataType
                };
                wfl.mag_dbs[bin] = mag_db; // write_val(freq_base_idx as isize, freq_osr_idx as isize, mag);
            }

            // self.wf.magsums[bin] += mag2;

            wfl.mags[bin] = mag2; // write_val(freq_base_idx as isize, freq_osr_idx as isize, mag);

            bin += 1;
            if bin >= self.wf.freq_bins() {
                break;
            }
            // dbg!(tmp_max);
        }
        wfl
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test() {
        // let input_vec = rfft_nfft_f.make_input_vec();
        // assert_eq!(input_vec.len(), nfft);
        // let rfft_output = rfft_nfft_f.make_output_vec();
        // dbg!("rfft_output.len: {}", rfft_output.len());
        // assert_eq!(rfft_output.len(), nfft/2 + 1);
    }
}
