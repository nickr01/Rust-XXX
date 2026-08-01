// use crate::constant;
// pub struct Subtractor {
//     protocol: &'static constant::Protocol,
//     runtime: &'static constant::Runtime,

//         // complex_fft: FftPlanner<f32>,
//     // fft_nmax_f: Arc<dyn Fft<f32>>,
//     // fft_nmax_i: Arc<dyn Fft<f32>>,

// }

// impl Subtractor {
//     pub fn new(
//         protocol: &'static constant::Protocol,
//         runtime: &'static constant::Runtime,
//     ) -> Subtractor {

//         // maybe we can unify these to share caches
//         // let mut complex_fft = FftPlanner::<f32>::new(); // single instance caches plans
//         // let fft_nmax_f = complex_fft.plan_fft_forward(nmax);
//         // let fft_nmax_i = complex_fft.plan_fft_inverse(nmax);

//         Subtractor {
//             protocol,
//             runtime,
//         }
//     }
// }

// nmax: usize, // sig buff size for time slot - used only for the subtractor

// Build a filter - NMAX long
// fn build_subtract_filter(&self) -> Result<Vec<Complex<f32>>, constant::XxxError> {
//     const NFILT: usize = 1400;

//     // Compute a Hann-like window directly into the real part of the
//     // first NFILT + 1 elements in the filter, accumulating the sum
//     // as we go.

//     let mut filter: Vec<Complex<f32>> = Vec::with_capacity(self.nmax);
//     filter.resize(self.nmax, Complex { re: 0.0, im: 0.0 });

//     let mut sum: f32 = 0.0;

//     for index in 0..NFILT
//     // for (int j = -NFILT / 2; j <= NFILT / 2; ++j)
//     // maybe should check against NFILT as i16 ?
//     {
//         let j: i16 = index as i16 - NFILT as i16/2;
//         let c: f32 = (PI * j as f32 / NFILT as f32).cos();
//         let value: f32 = c * c;

//         // let index: usize = (j + NFILT / 2) as usize;
//         filter[index] = Complex { re: value, im: 0.0 };
//         sum += value;
//     }

//     // // Now that we've got the sum, create actual complex numbers using
//     // // the normalized real values that we just populated and zero the
//     // // rest of the filter.

//     // std::fill(std::transform(filter.begin(),
//     //                          filter.begin() + NFILT + 1,
//     //                          filter.begin(),
//     //                          [sum](auto const value)
//     //                          {
//     //                              return std::complex<float>(value.real() / sum, 0.0f);
//     //                          }),
//     //           filter.end(),
//     //           ZERO);

//     {
//         let filter_end: usize = NFILT + 1;
//         for i in 0..filter_end {
//             filter[i] = Complex::new(filter[i].re / sum, 0.0);
//         }
//         filter.resize(filter_end, Complex { re: 0.0, im: 0.0 });
//         filter.resize(self.nmax, Complex { re: 0.0, im: 0.0 });
//     }

//     // // Shift to position the window.
//     // std::rotate(filter.begin(), // first
//     //             filter.begin() + NFILT / 2, // middle
//     //             filter.begin() + NFILT + 1); // last
//     // Performs a left rotation on a range of elements.
//     // Specifically, std::rotate swaps the elements in the range
//     // [first, last) in such a way that the elements in [first, middle)
//     // are placed after the elements in [middle, last)
//     // while the orders of the elements in both ranges are preserved.
//     // is -> middle, last, first
//     filter.rotate_left(NFILT/ 2);

//     // // Transform the filter into the frequency domain.
//     self.fft_nmax_f.process(&mut filter);

//     // fftwf_plan fftw_plan;
//     // {
//     //     std::lock_guard<std::mutex> lock(fftw_mutex);

//     //     fftw_plan = fftwf_plan_dft_1d(Mode::NMAX,
//     //                                   reinterpret_cast<fftwf_complex *>(filter.data()),
//     //                                   reinterpret_cast<fftwf_complex *>(filter.data()),
//     //                                   FFTW_FORWARD,
//     //                                   FFTW_ESTIMATE_PATIENT);

//     //     if (!fftw_plan)
//     //     {
//     //         throw std::runtime_error("Failed to create FFT plan");
//     //     }
//     // }

//     // fftwf_execute(fftw_plan);

//     // {
//     //     fftwf_destroy_plan(fftw_plan);
//     // }

//     // // Normalize the frequency domain representation.
//     {
//         let factor: f32 = 1.0/self.nmax as f32;
//         for i in 0..self.nmax {
//             filter[i] *= factor;
//         }
//     }

//     // std::transform(filter.begin(),
//     //                filter.end(),
//     //                filter.begin(),
//     //                [factor = 1.0f / Mode::NMAX](auto value)
//     //                {
//     //                    return value * factor;
//     //                });

//     Ok(filter)
// }

// // Generate a reference signal, based on the provided tone sequence and
// // base frequency. The output is a vector of complex values representing
// // the signal in the time domain.

// // std::vector<std::complex<float>> genjs8refsig(std::array<int, NN> const & itone, float               const   f0)
// fn genrefsig(&self, l0_tones: &Vec<u8>, f0: f32) -> Vec<Complex<f32>> {
//     // Precompute the base frequency contribution; full circle in
//     // radians, multipled by the base frequency, multiplied by the
//     // sampling interval, i.e., the time step between samples, which
//     // results in the base frequency phase increment. Start the
//     // phase accumulator off at zero.

//     let symp = self.protocol.symbol_period();
//     let nsps = self.runtime.target_input_sample_rate(); // 12000; // Mode::NSPS;
//     let bfpi: f32     = TAU * f0 * (1.0 / self.runtime.target_input_sample_rate().0 );

//     // std::vector<std::complex<float>> cref;
//     // cref.reserve(NN * Mode::NSPS);
//     let nn= self.protocol.total_symbols_nn(); // 79; // NN;
//     let cref_len = constant::ByteCount(nn.0 * nsps.0 as usize);

//     let mut cref: Vec<Complex<f32>> = Vec::with_capacity(cref_len.0);

//     let mut phi = 0.0;
//     // for (int i = 0; i < NN; ++i)
//     for i in 0..nn.0 {
//         // Compute phase increment for the tone; frequency offset is
//         // determined by the tone value.

//         let dphi: f32 = bfpi + TAU * ((l0_tones[i] * self.runtime.rx_freq_osr().0 as u8) as f32)/ nsps.0;

//         // Iterate over the samples per symbol to generate the time
//         // domain signal.

//         // for (std::size_t is = 0; is < Mode::NSPS; ++is)
//         for _i in 0..nsps.0 as usize {
//             // cref.push_back(std::polar(1.0f, phi));
//             cref.push(Complex::from_polar(1.0, phi));
//             // phi = std::fmod(phi + dphi, TAU);
//             phi = (phi + dphi) % TAU;
//         }
//     }
//     cref
// }

// // Subtract a JS8 signal - inplace in DD
// //
// // Measured signal  : dd(t)    = a(t)cos(2*pi*f0*t+theta(t))
// // Reference signal : cref(t)  = exp( j*(2*pi*f0*t+phi(t)) )
// // Complex amp      : cfilt(t) = LPF[ dd(t)*CONJG(cref(t)) ]
// // Subtract         : dd(t)    = dd(t) - 2*REAL{cref*cfilt}
// //
// // Important to note that dt can be negative here.
// fn subtract_candidate(&mut self,
//     // candidate: &Candidate,
//     detector: &Detector, cref: &Vec<Complex<f32>>, signal: &mut Vec<f32>, dt: f32
// ) {
//     if self.filter.len() == 0 {
//         self.filter = self.build_subtract_filter().unwrap();
//     }

//     signal.resize(self.nmax, 0.0);
//     // assert_eq!(signal.len(), self.nmax);

//     // let cref: &Vec<Complex<f32>> = &modem.signal;  // Needs cref to be complex - eg from genrefsig

//     let nstart: f32 = dt * self.runtime.target_input_sample_rate().0;
//     let cref_start = if nstart < 0.0 { -nstart as usize } else { 0 };
//     let dd_start   = if nstart > 0.0 { nstart as usize } else { 0 };
//     let size  = min(cref.len() - cref_start, self.nmax - dd_start);

//     // Populate complex filter with the conjugate of the reference signal.
//     let mut cfilt: Vec<Complex<f32>> = Vec::with_capacity(self.nmax);
//     for i in 0..size {
//         cfilt.push(signal[dd_start + i] * cref[cref_start + i].conj());
//     }

//     // Zero-fill the remainder, if any.
//     cfilt.resize(self.nmax, Complex::ZERO);

//     // FFT to the frequency domain.
//     self.fft_nmax_f.process(&mut cfilt);

//     // Apply the detector filter in the frequency domain.
//     for i in 0..self.nmax {
//         cfilt[i] *= self.filter[i];
//     }

//     // Inverse FFT to return to the time domain.
//     self.fft_nmax_i.process(&mut cfilt);

//     // Subtract the reconstructed signal. - correct! DD is signal is real
//     for i in 0..size {
//         // signal[dd_start + i] -= (cfilt[i] * modem.signal[signal_start + i]).re;
//         // signal[dd_start + i] -= 2.0 * (cfilt[i] * modem.signal[signal_start + i]).re;
//         signal[dd_start + i] -= 2.0 * (cfilt[i] * cref[cref_start + i]).re;
//     }
// }

//         JS8::encode(i3bit, Costas, message.data(), itone.data());

// // Subtract signal if needed.
// if lsubtract {
//     subtractjs8(genjs8refsig(itone, f1), xdt2);
// }

// if pass + 1 < self.runtime.subtracts() {
//     // subtract the successful decode
//     let crf_modem: constant::Modem = constant::Modem::new(
//         &constant::FT8,
//         &self.runtime,
//         freq_hz
//     );
//     let l3_out = crf_modem.l3_ecc_add(&message.codeword).unwrap();
//     let l2_out = crf_modem.l2_gray_encode(&l3_out).unwrap();
//     let l1_out = crf_modem.l1_sync_add(&l2_out).unwrap();

//     // need to get this direct approach to work?
//     // let cref: Vec<Complex<f32>> = self.genrefsig(&l1_out, freq_hz + 1.0 * 3.125);
//     // let time_adjust: f32 = 0.0;

//     // this is approx accurate - if use time_sec + 0.08125
//     // need to adjust for the 1 symbol pad at beginning
//     let l0_out = crf_modem.l0_gfsk_synth(&l1_out).unwrap();
//     let cref = hilbert(&l0_out);
//     // not sure yet why this needs to be adjusted by symbol_period/2
//     let time_adjust = self.protocol.symbol_period() / 2.0; // * self.protocol.ramp_symbols() as f32; //  / self.runtime.time_osr() as f32;

//     // self.subtract_candidate(
//     //     // &c,
//     //     &mut detector, &cref, &mut samples, time_sec + time_adjust
//     // );
// }

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test() {}
}
