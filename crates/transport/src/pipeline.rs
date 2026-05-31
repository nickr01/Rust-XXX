use crate::correlator;
use crate::decoder;
use crate::detector;
use crate::receiver;
use crate::rustxxx;
use crate::debug;

use audioadapter_buffers::direct::InterleavedSlice;
use ringbuf::traits::Consumer;
use ringbuf::traits::Observer;
use rubato::{Resampler, Fft, FixedSync, Indexing};
// use rustfft::num_traits::ops::saturating;

use std::collections::HashMap;

#[cfg(any(feature = "enable_rx", feature = "enable_tx", test))]
pub struct ResampleContext {
    pub from_channels: usize,
    pub from_rate: u32,
    pub resampler: Fft::<f32>,
}

#[cfg(any(feature = "enable_rx", test))]
pub struct Pipeline {
    pub receiver: receiver::Receiver,
    rfft_nfft_f: detector::DetectFFT,
    detector_input_bufs: detector::DetectorInputBuffs,
    correlator: correlator::Correlator,
    detector: detector::Detector,
    message_hash: decoder::DecodeHash,
    debug_portal: debug::DebugPortal,
}

#[cfg(any(feature = "enable_rx", test))]
impl Pipeline {
    const CHUNK_SIZE: usize = 2048;
    const SUB_CHUNK: usize = 1;
    const CHANNELS: usize = 1;
    const BUFLEN: usize = Pipeline::CHUNK_SIZE;

    pub fn new(
        protocol: &'static rustxxx::Protocol,
        runtime: &'static rustxxx::Runtime,
    ) -> Pipeline {
        let receiver = receiver::Receiver::new(protocol, runtime);
        let nfft = receiver.nfft;

        let mut real_fft = realfft::RealFftPlanner::<f32>::new(); 
        let rfft_nfft_f = real_fft.plan_fft_forward(nfft);

        let time_overlap =
            (runtime.target_input_sample_rate().0 / (protocol.baud_rate().0 * runtime.rx_symbol_osr().0 as f32)) as usize;
        dbg!(time_overlap);

        let buffers_required = nfft/time_overlap;
        dbg!(buffers_required);

        let mut detector_input_bufs: Vec<Vec<f32>>= vec![vec![0f32; nfft]; buffers_required];
        for (n, buf) in detector_input_bufs.iter_mut().enumerate() {
            // setup the buff phasing by presetting sizes
            let init_size = (buffers_required - n) * time_overlap;
            buf.resize(init_size, 0.0);
            assert_eq!(buf.len(), init_size);
        }

        let detector = detector::Detector::new(*runtime, *protocol,rustxxx::RepeatCount(nfft));
        let correlator = correlator::Correlator::new(protocol, runtime);

        assert!(detector.wf.time_bins() <= detector.wf.time_buf_capacity()); // capacity must be power of 2;

        let debug_portal = debug::DebugPortal::new(debug::DrawSize {
            width: detector.wf.freq_bins(), 
            height: detector.wf.time_bins() + detector.wf.symbol_pad() * runtime.rx_symbol_osr().0, 
        }, );

        Pipeline {
            receiver,
            rfft_nfft_f,
            detector_input_bufs,
            detector,
            correlator, 
            message_hash: HashMap::new(),
            debug_portal: debug_portal,
        }
    }

    pub fn resample_context (
        &self,        
        from_channels: usize,
        from_rate: u32,
    ) -> ResampleContext {
        let to_rate = self.receiver.runtime.target_input_sample_rate().0;
        dbg!(from_channels, from_rate, to_rate);

        let resampler = Fft::<f32>::new(
                from_rate as usize,
                to_rate as usize, 
                Pipeline::CHUNK_SIZE, 
                Pipeline::SUB_CHUNK, 
                Pipeline::CHANNELS, 
                FixedSync::Both
            ).unwrap();
 
        dbg!(resampler.resample_ratio());

        ResampleContext {
            resampler,
            from_channels,
            from_rate,
        }
    }

        // Dump mag4 spectrogram - should see separate blocks x freq_osr across x axis, and same for y
    pub fn draw_spectrogram(&mut self) {
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
        let wflines_iter = self.detector.wf.wflines().iter();
        for wfl in wflines_iter {
            let db_iter = wfl.mag_dbs.iter();
            for db in db_iter {
                spectr2.push(*db);
            }
        }

        let spectrogram_height = self.detector.wf.wflines().len();
        if spectrogram_height > 0 {
            use crate::debug::DrawSize;

            debug::plot_spectrogram_to_buffer(
                self.debug_portal.buf_as_mut(), 
                &spectr2,
                DrawSize{ width: spectr2.len()/spectrogram_height, height: spectrogram_height },
            );
        }
    }

    pub fn update_spectrogram(&mut self) {
        self.draw_spectrogram();
        self.debug_portal.update();
    }

    fn write_sample(&mut self, sample: f32) -> Result<(), rustxxx::XxxError> {
        let buf_consumed = self.receiver.load_sample_into_waterfall_lines(
            sample,
            &mut self.rfft_nfft_f,
            &mut self.detector_input_bufs,
            &mut self.detector, 
            &mut self.correlator,
            &mut self.message_hash
        );
        // if buf_consumed > 0 {
        //     self.update_spectrogram();
        // }
        Ok(())
    }

    pub fn continue_run(&self) -> bool {
        self.debug_portal.continue_run()
    }

    pub fn write_sample_buffer(
        &mut self,
        reader: &mut rustxxx::ThreadedAudioReader,
        resample_context: &mut ResampleContext,
    ) -> Result<Vec<Vec<u8>>, rustxxx::XxxError> {
        {
            let mut mono_samples = Vec::new();
            if let Ok(mut guard) = reader.try_lock() {
                let reader = guard.as_mut();

                dbg!(reader.occupied_len());
                
                // MUST rebuild the iterator each loop - see next() documentation
                let count_to_load = reader.occupied_len() & !1; // force even consumption
                assert_eq!(count_to_load & 1, 0);
                let planned_load = Pipeline::BUFLEN * resample_context.from_channels;

                if count_to_load >= planned_load {
                    let count_to_load = planned_load; // coerce to controlled buffer size - maybe not necessary
                    let samples_iter = reader.pop_iter();
                    {
                        let mut count = 0;
                        for sample in samples_iter {
                            if resample_context.from_channels == 1 || count & 1 == 0 {
                                mono_samples.push(sample);
                            };
                            count += 1;
                            if count >= count_to_load {
                                break;
                            }
                        }
                    }
                }
            }

            // dbg!(count_to_load, mono_samples.len());
            // now do the sample_rate if necessary
            let samples_at_new_rate = if resample_context.from_rate == self.receiver.runtime.target_input_sample_rate().0 as u32 {
                mono_samples
            } else {
                // let audio_clip = vec![0.0; 2*10000];

                // wrap it with an InterleavedSlice Adapter
                let nbr_input_frames = mono_samples.len(); // audio_clip.len() / 2;
                let input_adapter = InterleavedSlice::new(&mono_samples, 1, nbr_input_frames).unwrap();

                // create a buffer for the output
                let out_len = (mono_samples.len() as f64 * resample_context.resampler.resample_ratio()) as usize;
                // dbg!(mono_samples.len(), out_len);
                let mut outdata: Vec<f32> = vec![0f32;out_len];
                let outdata_capacity = outdata.len();
                let mut output_adapter =
                    InterleavedSlice::new_mut(
                        &mut outdata, 1, outdata_capacity
                    ).unwrap();

                // Preparations
                let mut indexing = Indexing {
                    input_offset: 0,
                    output_offset: 0,
                    active_channels_mask: None,
                    partial_len: None,
                };

                let mut input_frames_left = nbr_input_frames;
                let mut input_frames_next = resample_context.resampler.input_frames_next();

                // Loop over all full chunks.
                // There will be some unprocessed input frames left after the last full chunk.
                // see the `process_f64` example for how to handle those
                // using `partial_len` of the indexing struct.
                // It is also possible to use the `process_all_into_buffer` method
                // to process the entire file (including any last partial chunk) with a single call.
                while input_frames_left >= input_frames_next {
                    let (frames_read, frames_written) = resample_context.resampler
                        .process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
                        .unwrap();

                    indexing.input_offset += frames_read;
                    indexing.output_offset += frames_written;
                    input_frames_left -= frames_read;
                    input_frames_next = resample_context.resampler.input_frames_next();
                }
                outdata
            };

            // this is occasionally expensive triggering decode processing into message_hash
            for sample in samples_at_new_rate {
                self.write_sample(sample)?;
            }

            // return the message_hash content if already returned
            // mark as reported
            // clean the message_hash - remove older than message length secs
        };

        let mut ret: Vec<Vec<u8>> = Vec::new();
        // for codeword in self.message_hash.keys() {
        //     ret.push(codeword.clone());
        // }

        Ok(ret)
    }

    // pub fn report_results(&self) -> Result<(), rustxxx::XxxError>
    // {
    //     let messages = &self.message_hash;
    //     dbg!(messages.len());
    //     dbg!(self.receiver.start_time.elapsed());

    //     let mut msg_dfs: Vec<decoder::MessageDf> = Vec::new();

    //     for msg_key in messages.keys() {
    //         msg_dfs.push(messages.get(msg_key).unwrap().clone())
    //     }

    //     msg_dfs.sort_by_key(|b| b.freq_hz.0 as i32);
    //     // msg_dfs.sort_by_key(|b| std::cmp::Reverse(b.freq_hz as i32));

    //     // let mut i = 1;
    //     // for df in msg_dfs {            
    //     for (i, df) in (1..).zip(msg_dfs) {            
    //         dbg!(
    //             i,
    //             (df.freq_hz.0 * 10.0).round() / 10.0,
    //             (df.time_secs.0 * 10.0).round() / 10.0,
    //             df.c_score,
    //             df.text
    //         );
    //         // i += 1;
    //     }
    //     Ok(())
    // }
}
