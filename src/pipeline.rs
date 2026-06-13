use crate::correlator;
use crate::decoder;
use crate::detector;

#[cfg(any(feature = "enable_rx", test))]
use crate::receiver;

use crate::error;
use crate::types;
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
pub const RX_IN_BUFLEN: usize = Pipeline::CHUNK_SIZE;

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
    const CHUNK_SIZE: usize = 8192 ; // 2048 is too little to keep up with 48K audio stream
    const SUB_CHUNK: usize = 1;  // maybe can tune this
    const CHANNELS: usize = 1;

    pub fn new(
        protocol: &'static types::Protocol,
        runtime: &'static types::Runtime,
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

        let detector = detector::Detector::new(*runtime, *protocol,types::RepeatCount(nfft));
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
        let wflines_iter = self.detector.wf.wflines().iter().rev();
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

    fn write_sample(&mut self, sample: f32) -> Result<(), error::XxxError> {
        let _buf_consumed = self.receiver.load_sample_into_waterfall_lines(
            sample,
            &mut self.rfft_nfft_f,
            &mut self.detector_input_bufs,
            &mut self.detector, 
            &mut self.correlator,
            &mut self.message_hash
        );
        Ok(())
    }

    pub fn continue_run(&self) -> bool {
        self.debug_portal.continue_run()
    }

    pub fn write_mono_sample_buffer(
        &mut self,
        mono_samples: &[f32; RX_IN_BUFLEN],
        resample_context: &mut ResampleContext,
    ) -> Result<Vec<types::Message>, error::XxxError> {
        {
            // let planned_load = Pipeline::sample_buf_size(resample_context); // BUFLEN * resample_context.from_channels;
            // assert_eq!(planned_load & 1, 0); // even
            // assert_eq!(mono_samples.len(), planned_load);
            // if audio_in.len() & !1 >= planned_load { // force even consumption
            {
                // todo!("remove this step - expect mono");
                // let mut mono_samples = Vec::with_capacity(Pipeline::BUFLEN);
                // if resample_context.from_channels == 1 {
                //     mono_samples = mono_in.clone();
                // } else {
                //     for (i, sample) in mono_in.iter().enumerate() {
                //         if resample_context.from_channels == 1 || i & 1 == 0 {
                //             mono_samples.push(*sample);
                //         };
                //     };
                // }

                // assert_eq!(reader.occupied_len(),  count_to_load - planned_load);

                // dbg!(count_to_load, mono_samples.len());
                // now do the sample_rate if necessary
                let samples_at_new_rate = if resample_context.from_rate == self.receiver.runtime.target_input_sample_rate().0 as u32 {
                    Vec::from(mono_samples)
                } else {
                    // let audio_clip = vec![0.0; 2*10000];

                    // wrap it with an InterleavedSlice Adapter
                    let nbr_input_frames = mono_samples.len(); // audio_clip.len() / 2;
                    let input_adapter = InterleavedSlice::new(mono_samples, 1, nbr_input_frames).unwrap();

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
            }
        };

        let stale_time =
            types::Secs(
                (self.detector.wf.time_base().0 as f32 / self.receiver.runtime.rx_symbol_osr().0 as f32) 
                * self.receiver.protocol.symbol_period().0
                - self.receiver.protocol.total_frame_time().0
            );

        // report the current undelivered msgs and check stale status
        let mut delivery_msgs: Vec<types::Message> = Vec::new();
        let mut stale_msgs: Vec<types::Message> = Vec::new();

        for msg in self.message_hash.values() {
            if !msg.is_delivered() {
                let mut msg = msg.clone();
                msg.set_delivered();
                delivery_msgs.push(msg);
            }
            if msg.is_stale(stale_time) {
                stale_msgs.push(msg.clone())
            }
        }

        for msg in delivery_msgs.iter() {
            dbg!("Updating msg DELIVER flag");
            self.message_hash.remove(msg.key());         
            assert!(msg.is_delivered());
            self.message_hash.insert(msg.key().clone(), msg.clone());         
        }

        for msg in stale_msgs.iter() {
            dbg!("Deleting STALE msg");
            assert!(msg.is_delivered());
            self.message_hash.remove(msg.key());         
        }

        Ok(delivery_msgs)
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
