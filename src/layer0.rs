// use std::f32::consts::PI;
// use std::f32::consts::TAU;
// Layer0 Audio - gfsk
// use std::result::*;

use crate::error;
use crate::types;

impl types::Modem {
    /// Computes a GFSK smoothing pulse.
    /// The pulse is theoretically infinitely long, however, here it's truncated at 3 times the symbol length.
    /// This means the pulse array has to have space for 3*n_spsym elements.
    /// @param[in] n_spsym Number of samples per symbol
    /// @param[in] b Shape parameter (values defined for XX8/FT4)
    /// @param[out] pulse Output array of pulse samples
    ///
    #[cfg(any(feature = "enable_tx", test))]
    pub fn _gfsk_pulse(&self, n_spsym: usize, pulse: &mut [f32]) {

        let mut symbol_bt = self.protocol()._symbol_bt();
        // Generate an error function with a length three times the symbol length using the Bt product 2
        if cfg!(feature = "disable_gfsk") {
            symbol_bt = 99.0;
        }

        ///< symbol smoothing filter bandwidth factor (BT
        const GFSK_CONST_K: f32 = 5.336446f32;

        for (i, p) in pulse.iter_mut().enumerate().take(self.protocol().token_bits().0 * n_spsym) {
            let t = i as f32 / n_spsym as f32 - 1.5;
            let arg1 = GFSK_CONST_K * symbol_bt * (t + 0.5);
            let arg2 = GFSK_CONST_K * symbol_bt * (t - 0.5);
            *p = (libm::erff(arg1) - libm::erff(arg2)) / 2.0;
        }
    }

    /// Synthesize waveform data using GFSK phase shaping.
    /// The output waveform will contain n_sym symbols.
    /// @param[in] symbols Array of symbols (tones) (0-7 for XX8)
    /// @param[in] n_sym Number of symbols in the symbol array
    /// @param[in] f0 Audio frequency in Hertz for the symbol 0 (base frequency)
    /// @param[in] symbol_bt Symbol smoothing filter bandwidth (2 for XX8, 1 for FT4)
    /// @param[in] symbol_period Symbol period (duration), seconds
    /// @param[in] signal_rate Sample rate of synthesized signal, Hertz
    /// @param[out] signal Output array of signal waveform samples (should have space for n_sym*n_spsym samples)
    ///
    #[cfg(any(feature = "enable_tx", test))]
    fn _gfsk_synth(&self, l0_tones: &[u8]) -> Vec<f32> {
        // let sym_period = self.protocol.symbol_period();
        let n_spsym = (0.5 + self.runtime()._target_output_sample_rate().0) as usize; // Samples per symbol
        
        let n_wave = self.protocol().total_symbols_nn().0 * n_spsym; // Number of output samples
        let mut signal = Vec::with_capacity(n_wave);

        // let symbols = l0_tones;
        let f0 = self.freq_hz().unwrap();

        // let n_spsym = (0.5 + signal_rate * symbol_period) as usize; // Samples per symbol
        // let n_wave = n_sym * n_spsym; // Number of output samples

        let n_sym = self.protocol().total_symbols_nn();
        const MAGIC_NUM: f32 = 0.5;
        let n_spsym = (MAGIC_NUM + self.runtime()._target_output_sample_rate().0) as usize; // Samples per symbol
        let n_wave = self.protocol().total_symbols_nn().0 * n_spsym; // Number of output samples

        let hmod = 1.0f32;

        // Compute the smoothed frequency waveform.
        // Length = (nsym+2)*n_spsym samples, first and last symbols extended
        let dphi_peak = std::f32::consts::TAU * hmod / n_spsym as f32;
        let mut dphi = Vec::new();

        // Shift frequency up by f0
        for _ in 0..(n_wave + 2 * n_spsym) {
            dphi.push(std::f32::consts::TAU * f0 / self.runtime()._target_output_sample_rate().0);
        }

        let mut pulse = vec![0.0; self.protocol().token_bits().0 * n_spsym];

        self._gfsk_pulse(n_spsym, &mut pulse);

        // if cfg!(not(feature = "disable_gfsk_plot")) {
        //     plot_graph(
        //         "./out/gauss-envelope.png",
        //         "GFSK Phase Envelope",
        //         &pulse,
        //         0, pulse.len(),
        //         0.0, 1.0,
        //     );
        // }

        for (i, sym) in l0_tones.iter().enumerate().take(n_sym.0) {
            let ib = i * n_spsym;
            for j in 0..self.protocol().token_bits().0 * n_spsym {
                dphi[j + ib] += dphi_peak * (*sym as f32) * pulse[j];
            }
        }

        // if cfg!(not(feature = "disable_gfsk_plot")) {
        //     plot_graph(
        //         "./out/tones.png", 
        //         "GFSK Tones", 
        //         &dphi, 
        //         0, 16000, 
        //         0.625, 0.65
        //     );
        // }

        // Add dummy symbols at beginning and end with tone values equal to 1st and last symbol, respectively
        for j in 0..(2 * n_spsym) {
            dphi[j] += dphi_peak * pulse[j + n_spsym] * l0_tones[0] as f32;
            dphi[j + n_sym.0 * n_spsym] += dphi_peak * pulse[j] * l0_tones[n_sym.0 - 1] as f32;
        }

        // Calculate and insert the audio waveform
        let mut phi = 0.0f32;
        for k in 0..n_wave {
            // Don't include dummy symbols
            signal.push(phi.sin());
            phi = libm::fmodf(phi + dphi[k + n_spsym], std::f32::consts::TAU);
        }

        // Apply envelope shaping to the first and last symbols
        if cfg!(not(feature = "disable_gfsk_ramp")) {
            let n_ramp = n_spsym / 8;
            for i in 0..n_ramp {
                let env = (1.0 - (std::f32::consts::TAU * i as f32 / (2.0 * n_ramp as f32)).cos()) / 2.0;
                signal[i] *= env;
                signal[n_wave - 1 - i] *= env;
            }
        }

        signal
    }

    #[cfg(any(feature = "enable_rx", test))]
    fn _gfsk_decode(&self, _signal: &Vec<f32>) -> Result<Vec<u8>, error::XxxError> {
        // call monitor and receiver
        Err(error::XxxError::_ToDo)
    }

    // These are the action stubs
    #[cfg(any(feature = "enable_tx", test))]
    pub fn _l0_gfsk_synth(&self, l0_tones: &[u8],
        // l0_tones: &[u8; XXX.nn()]
    ) -> Result<Vec<f32>, error::XxxError>{
        Ok(self._gfsk_synth(l0_tones))
    }

    #[cfg(any(feature = "enable_rx", test))]
    pub fn _l0_gfsk_undo(&self, 
        signal: &Vec<f32>,
        // l0_tones: &[u8; XXX.nn()]
    ) ->Result<Vec<u8>, error::XxxError> {
        // let bad = [1u8; XXX.nn()];
        self._gfsk_decode(signal)
    }

    #[cfg(any(test))]
    pub fn l0_outbound(&self, ttl: isize, l0_tones: &Vec<u8>) -> Result<Vec<u8>, error::XxxError> {
        let _signal = self._l0_gfsk_synth(l0_tones)?;

        if ttl == 0 {
            self.l1_inbound(&l0_tones)  // loopback
        } else {
            let empty: Vec<u8> = Vec::new();
            Ok(empty)
        }
    }

    #[cfg(any(test))]
    pub fn _l0_inbound(&self, signal: &Vec<f32>) ->Result<Vec<u8>, error::XxxError> {
        let l0_tones = self._l0_gfsk_undo(signal)?;
        self.l1_inbound(&l0_tones)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use crate::test_support;

    // Https://docs.rs/jack/latest/jack/

    // const _L0RT0: [u8; rustxxx::FT8.total_symbols_nn().0] = [
    //         3, 1, 4, 0, 6, 5, 2, 5, 5, 5, 
    //         5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 
    //         5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 
    //         5, 5, 5, 5, 5, 5, 3, 1, 4, 0, 
    //         6, 5, 2, 5, 5, 5, 5, 5, 5, 5, 
    //         5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 
    //         5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 
    //         5, 5, 3, 1, 4, 0, 6, 5, 2
    // ];
    // const L0RT1: [u8; rustxxx::FT8.total_symbols_nn().0] = [
    //         3, 1, 4, 0, 6, 5, 2, 0, 1, 2,
    //         3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 
    //         5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 
    //         7, 0, 1, 2, 3, 4, 3, 1, 4, 0, 
    //         6, 5, 2, 5, 6, 7, 0, 1, 2, 3, 
    //         4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 
    //         6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 
    //         0, 1, 3, 1, 4, 0, 6, 5, 2
    // ];

    // fn test_roundtrip(modem: &mut rustxxx::Modem, l1_tones: &Vec<u8>)
    // {
    //     let _num_samples =
    //         (0.5 + modem.protocol.total_symbols_nn().0 as f32 * rustxxx::TEST_FT8_RUNTIME._target_output_sample_rate().0) as usize;
    //     // let num_silence = ((XXX.slot_time() *XXX.sample_rate() as f32) as usize -num_samples) /2;
    //     // let mut samples = vec![0.0; num_samples];

    //     let l0_tones = l1_tones.clone();
    //     modem._freq_hz = rustxxx::TEST_FREQUENCY;
    //     let _signal = modem._gfsk_synth(&l0_tones);

    // //     #[cfg(feature = "gfsk_dump_wav")]
    // //     {
    // //         use wav_io::{*, header::WavHeader, header::SampleFormat, header::WavData};
    // //         use std::fs::File;
    // //         modem._dump_wav(&_signal);
    // //     }
    // }

    // #[test]
    // fn test_layer0() {
    //     let mut modem: rustxxx::Modem = rustxxx::Modem::new(
    //         &rustxxx::TEST_PROTOCOL, 
    //         &rustxxx::TEST_FT8_RUNTIME, 
    //         rustxxx::TEST_FREQUENCY
    //     );
    //     test_roundtrip(&mut modem, &L0RT1.to_vec());
    // }
}