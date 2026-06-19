// use std::sync::mpsc::Receiver;

use anyhow::{
    // Context, 
    Result
}; // - for user level

// use monitor::Waterfall;
// use ringbuf::storage::Heap;
// use ringbuf::SharedRb;
// use std::env;
// use std::fs::File;
// use std::sync::{Arc, Mutex};
// use std::sync::mpsc::*;

// use std::thread::{JoinHandle, yield_now};
// use std::thread;
// use wav_io::header::*;
// use wav_io::*;

// needed for Traits
use clap::Parser;

use cpal::{
    StreamConfig, 
    // SupportedStreamConfig
};
// use cpal::{Sample, SupportedStreamConfig};
use cpal::traits::{
    // HostTrait, 
    DeviceTrait, 
    StreamTrait
};

// use cpal::{StreamConfig, SupportedStreamConfig};
// use cpal::{Sample, SupportedStreamConfig};
// use cpal::traits::{ HostTrait, DeviceTrait, 
//     // StreamTrait
// };

use ft8_message;

use rustxxx::cpal_helper;
// use rustxxx::debug;
// #[cfg(any(feature = "enable_rx", test))]
// use rustxxx::debug::DebugWindow;
use rustxxx::rx_pipeline;
use rustxxx::types::*;

// use ringbuf::{traits::*, HeapRb, SharedRb};
// use this for the waterfalll pipe
use ringbuf::traits::Consumer;
use ringbuf::traits::Producer;
use ringbuf::traits::Split;
// use ringbuf::wrap::caching::Caching;
// use ringbuf::CachingCons;

// use this for the audio pipes
//use ringbuffer_spsc::ringbuffer;

// mod rustxxx;
// mod cpal_helper;
// mod crc;
// mod decode;
// mod encode;
// mod gfsk;
// mod ldpc;
// mod monitor
// mod rx_blocks;
// mod pipeline;
// mod receiver;
// mod detector;
// mod waterfall;
// mod debug;
// mod correlator;
// mod decoder;
// mod subtractor;
// mod text;
// mod candidate;
// mod pack_ft8;
// mod unpack_ft8;

// mod crc1;
// mod generator;
// mod parity;

// mod test_utils;
// mod layer0; // Audio
// mod layer1; // sync
// mod layer2; // gray
// mod layer3; // Ecc
// mod layer4; // Crc
// mod layer5; // Top - UCP-API or FT8 app layer connect here

// use constant::*;
// use receiver::*;

// use decode::*;
// use encode::*;
// use gfsk::synth_gfsk;
// use monitor::Candidate;
// use monitor::Monitor;
// use pack_ft8::*;
// use unpack_ft8::*;

// #[cfg(feature = "gfsk_dump_wav")]
// pub fn _dump_wav(&self, signal: &Vec<f32>) {
//     // 	let attn = args[2].parse::<f32>().expected();
//     let attn = 10.0_f32.powf(0.0/20.0);

//     let samples = signal.iter().map(|x| *x *attn).collect::<Vec<_>>();

//     let mut header = WavHeader::new_mono();

//     header.sample_format = SampleFormat::Float;
//     header.channels = 1;
//     header.sample_rate = TEST_FT8_RUNTIME.sample_rate() as u32;
//     header.bits_per_sample = TEST_FT8_RUNTIME._bit_depth() as u16;

//     let mut file_out = File::create("./out/gfsk_dump.wav").expect();
//     writer::to_file(&mut file_out, &WavData::new(header, samples.clone())).expect();
// }
        // // #[cfg(feature = "gfsk_dump_wav")]
        // self._dump_wav(&_signal);

        // if cfg!(not(feature = "disable_gfsk_plot")) {
        //     plot_graph(
        //         "./out/signal.png", 
        //         "GFSK Signal", 
        //         &signal, 
        //         0, 500, 
        //         -1.5, 1.5
        //     );
        // }

pub type AudioSampleBuffer = ringbuf::SharedRb<ringbuf::storage::Heap<f32>>; // SharedRb<ringbuf::storage::Heap<f32>>;

pub type AudioBufWriter = ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, true, false>; 
pub type AudioBufReader = ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, false, true>;

// pub type AudioBufReader = std::sync::Arc<std::sync::Mutex<ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, false, true>>>;
// pub type AudioBufWriter = std::sync::Arc<std::sync::Mutex<ringbuf::wrap::caching::Caching<std::sync::Arc<ringbuf::SharedRb<ringbuf::storage::Heap<f32>>>, true, false>>>;


pub const FT8: Protocol = Protocol::new(
    Secs(0.16),
    Secs(15.0),
    true,
    BitCount(3),
    SymbolCount(58),
    SymbolCount(79),             // Total channel symbols (FT8_NS + FT8_ND)
    SymbolCount(7),     // sync group length
    RepeatCount(3),        // Number of sync groups
    SymbolCount(0),
    SymbolCount(36),    // Offset between sync groups
    [3, 1, 4, 0, 6, 5, 2],    //　Costas array
    BitCount(174),        // Number of bits in the encoded message (payload with LDPC checksum bits)
    BitCount(91),         // Number of payload bits (including CRC)
    [0, 1, 3, 2, 5, 6, 4, 7],
    [0, 1, 3, 2, 6, 4, 5, 7],
    CrcParams::new(BitCount(5),BitMap(0x2757), BitCount(14), 0, 0),
    2.0f32,
    SymbolCount(1),
);

pub const FT8_RUNTIME: Runtime = Runtime::new(
    // should be indep of bandwidth and freq_osr but not there yet
    Hz(6000.0),  // this is the real design layer - app layer can chose a portion often 250-2500
    BitCount(32),
    OverSampleMultiplier(4), // 4
    OverSampleMultiplier(2), // 2
    // detector_underload_divisor: RepeatCount(1), // 2 as per WB2FKO doc
    1.0, // 0.4, // 10,
    RepeatCount(1),
    RepeatCount(20),
    false, // true, 
    // subtracts: RepeatCount(1),
    WindowFunction::_Hann,  // Hann in the FT8_lib c code, or Blackman
);

#[derive(clap::Parser, std::fmt::Debug)]
#[command(version, about = "Rust-XXX FT8-like modem testbed", long_about = None)]
struct Opt {
    /// The audio input device to use.
    #[arg(short, long, default_value = "")]
    input_device: Option<String>,

    // /// The audio input file to use. 
    // #[arg(long)]
    // input_file: Option<String>,

    /// The audio output device to use.
    #[arg(short, long, default_value = "")]
    output_device: Option<String>,

    // /// The audio input file to use. 
    // #[arg(long)]
    // output_file: Option<String>,

    #[arg(short, long, default_value = "true")]
    loop_back: Option<bool>,

    //  How long to record, in seconds
    // #[arg(long, default_value_t = 15)]
    // duration: u64,

    // Slowest js8speed in test. Determines time modulus.
    // #[arg(short, long, default_value = "normal")]
    // speed: Speed,
}

// #[cfg(any(feature = "enable_rx", test))]
// fn do_audio_file_input(
//     runtime: rustxxx::types::Runtime, 
//     input_buff_writer: &mut AudioBufWriter, 
//     input_file: &String,
//     from_channels: &mut usize,
//     from_rate: &mut u32
// ) -> Result<Option<cpal::Stream>, anyhow::Error>
// {
//     // const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", input_file, ".wav");
//     // let spec = wav_spec_from_config(&config);
//     // let writer = hound::WavWriter::create(PATH, spec)?;
//     // let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));

//     // Input from file
//     let input_wav = std::fs::File::open(&input_file)
//         .context(format!("Cannot open input wav file {}", input_file))?;

//     let (header, signal) = wav_io::read_from_file(input_wav)
//         .context(format!("Cannot read from wav file {}", input_file))?;

//     dbg!(&header);

//     *from_channels = header.channels as usize;
//     *from_rate = header.sample_rate as u32;

//     // if header.channels != runtime.channels().0 as u16 {
//     //     let new_channels = runtime.channels().0 as u16;
//     //     dbg!(new_channels);
//     //     signal = wav_io::utils::stereo_to_mono(signal);
//     //     header.channels = new_channels;
//     // }

//     // dbg!(runtime.target_input_sample_rate());
//     // {
//     //     let target_sample_rate = runtime.target_input_sample_rate().0 as u32;
//     //     if header.sample_rate != target_sample_rate {
//     //         dbg!(header.sample_rate, target_sample_rate);
//     //         signal = wav_io::resample::linear(
//     //             signal, 
//     //             runtime.channels().0 as u16, 
//     //             header.sample_rate, 
//     //             target_sample_rate
//     //         );
//     //         header.sample_rate = target_sample_rate;
//     //     }
//     // }

//     // dbg!(runtime.subtracts());

//     // let mut file_out = File::create("./out/resampled.wav").expect();
//     // writer::to_file(&mut file_out, &WavData::new(header, samples.clone())).expect();

//     dbg!(signal.len());

//     dbg!(
//         runtime.rx_symbol_osr(),
//         runtime.rx_freq_osr()
//     );

//     // let input_buf = ringbuf::HeapRb::<f32>::new(signal.len());
//     // for testing we'll preload a buffer block
//     // if let Ok(mut guard) = input_buff_writer.try_lock() 
//     {
//         // let input_buff_writer = guard.as_mut();
//         for sample in signal.iter() {
//             input_buff_writer.try_push(*sample).expect("input_buf overrun");
//         }
//     }
    
//     Ok(None)
// }

// fn do_audio_file_output 
//     if let Some(_output_file) = opt.output_file {
        // encode arm
        // The WAV file we're recording to.
        // const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", output_file, ".wav");
        // let spec = wav_spec_from_config(&config);
        // let writer = hound::WavWriter::create(PATH, spec)?;
        // let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));

        // panic!();
        // Generate XX8 symbols and GFSK modulated samples.
        // let frequency = args[1].parse::<f32>().expect();
		// let attn = args[2].parse::<f32>().expect();
		// let attn = 10.0_f32.powf(attn/20.0);

        // if pack77(&args[3], &mut packed) < 0 {
        //     dbg!("Cannot parse message! {}", &args[1]);
        //     return;
        // }

        // xx8_encode(&packed, &mut tones);

        // print!("FSK tones: ");
        // for t in tones.iter() {
        //     print!("{} ", t);
        // }
        // dbg!();

        // let num_samples =
        //     (0.5 + XXX.nn() as f32 *XXX.symbol_period() *CONFIG.sample_rate as f32) as usize;
        // let num_silence = ((XXX.slot_time() *CONFIG.sample_rate as f32) as usize -num_samples) /2;

        // samples = vec![0.0; num_samples];

        // synth_gfsk(
        //     &tones,
        //     XXX.nn(),
        //     frequency,
        //     XXX.symbol_bt(),
        //     XXX.symbol_period(),
        //     CONFIG.sample_rate as f32,
        //     &mut samples,
        // );

        // let mut silence_before = vec![0.0; num_silence];
        // let mut silence_after = vec![0.0; num_silence];

        // silence_before.append(&mut samples);
        // silence_before.append(&mut silence_after);
        // samples = silence_before;

        // samples = samples.iter().map(|x| *x *attn).collect::<Vec<_>>();
        // header.sample_format = SampleFormat::Float;
        // header.sample_rate = CONFIG.sample_rate;
        // header.channels = 1;
        // header.bits_per_sample = 32;

        // let mut file_out = File::create("./resampled.wav").expect();
        // writer::to_file(&mut file_out, &WavData::new(header, samples.clone())).expect();
    // };

// fn setup_device_input(
//     _runtime: constant::Runtime, input_buff:& mut constant::InputBuffer, host: cpal::Host, input_device_name: String
// ) -> Result<(), anyhow::Error> {

//     Ok(())
// }

// type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

#[cfg(any(feature = "enable_tx", test))]
fn tx_main(
    output_device: &Option<String>,
    runtime: &'static rustxxx::types::Runtime,
) -> Result<(), anyhow::Error> {
    dbg!("Sender start");

    let host = cpal::default_host();
    
    let mut _audio_output_to_channels = 0; 
    let mut _audio_output_to_rate = 0;
    let audio_output_buffer: AudioSampleBuffer = ringbuf::HeapRb::<f32>::new(rustxxx::types::AUDIO_OUTPUT_BUFSIZE);
    let audio_err_callback = move |err| {
        eprintln!("an error occurred on audio output stream: {err}");
    };

    let (mut _audio_output_buff_writer, mut audio_output_buff_reader) = audio_output_buffer.split();

    let _audio_output_stream = if let Some(audio_output_device_name) = &output_device {
        dbg!(&audio_output_device_name);

        let (audio_output_device, audio_output_config) = cpal_helper::get_audio_output_device_default_config(&host, &audio_output_device_name)?;
        dbg!(&audio_output_config);

        let audio_output_from_channels = 1;
        let audio_output_from_rate = runtime.target_input_sample_rate().0 as u32;

        _audio_output_to_channels = audio_output_config.channels() as usize;
        _audio_output_to_rate = audio_output_config.sample_rate();

        dbg!(audio_output_from_channels, _audio_output_to_channels, audio_output_from_rate, _audio_output_to_rate);

        fn audio_output_data_callback(output: &mut [f32], reader: &mut AudioBufReader) {
            // if let Ok(mut guard) = reader.try_lock() 
            {
                // let reader = guard.as_mut();
                let mut output_fell_behind = false;
                for sample in output {
                    *sample = match reader.try_pop() {
                        Some(s) => s,
                        None => {
                            output_fell_behind = true;
                            0.0
                        }
                    };
                }
                if output_fell_behind {
                    // dbg!("output stream fell behind");
                }
            }
        }

        let audio_output_config: StreamConfig = audio_output_config.into();  
        
        // this will spawn a system thread that runs the callback
        // callback should be lightweight
        // NB when ownership of 'stream' is lost then it is shutdown!!
        let audio_output_stream = audio_output_device.build_output_stream(
            &audio_output_config,
            move |data, _: &_| audio_output_data_callback(
                data, 
                &mut audio_output_buff_reader,
            ),
            audio_err_callback,
            None, 
        ).expect("Cannot create audio output stream");

        audio_output_stream.play().expect("Cannot play audio output stream");
        Some(audio_output_stream)
    } else {
        None
    };

    match _audio_output_stream {
        Some(_stream) => {
        },
        None => {}
    }

    Ok(())
}

#[cfg(any(feature = "enable_rx", test))]
fn rx_main(
    input_device: &Option<String>,
    runtime: &'static rustxxx::types::Runtime,
) -> Result<(), anyhow::Error> {
    dbg!("Receiver start");

    let host = cpal::default_host();
    
    // // Set up the CPAL output device and stream with the default output config.
    // let output_device = if let Some(output_device) = opt.output_device {
    //     let id = &output_device.parse().expect("failed to parse input device id");
    //     host.device_by_id(id)
    // } else {
    //     host.default_output_device()
    // }
    //     .expect("failed to find an output device");
    // dbg!("Output device: {}", output_device.id()?);

    // let output_config = if output_device.supports_output() {
    //     Some(output_device.default_output_config())
    // } else {
    //     None
    // };
    // dbg!("output_config {:?}", output_config);

    // let config = if device.supports_input() {
    //     device.default_input_config()
    // } else {
    //     device.default_output_config()
    // }
    // .expect("Failed to get default input/output config");
    // dbg!("Default input/output config: {config:?}");

    // let args: Vec<String> = std::env::args().collect();

    // let mut signal: Vec<f32> = Vec::new();
    // let mut header = WavHeader::new();
    // let mut packed = [0u8; TEST_PROTOCOL.ldpc_k_bytes()];
    // let mut tones = [0usize; TEST_PROTOCOL.nn()];

    let audio_err_callback = move |err| {
        eprintln!("an error occurred on audio input stream: {err}");
    };

    // these get init by the device init blocks
    // pipeline needs to know these as conversions happen in the pipeline i/o
    // which leaves the audio thread callbacks as light as possible
    let mut audio_input_from_channels = 0; 
    let mut audio_input_from_rate = 0;

    let audio_input_buffer: AudioSampleBuffer = ringbuf::HeapRb::<f32>::new(rustxxx::types::AUDIO_INPUT_BUFSIZE);

    let (mut audio_input_buff_writer, mut audio_input_buff_reader) = audio_input_buffer.split();
    // let mut audio_input_buff_writer: rustxxx::rustxxx::ThreadedAudioBufWriter = std::sync::Arc::new(std::sync::Mutex::new(_audio_input_buff_writer));
    // let mut audio_input_buff_reader: rustxxx::rustxxx::ThreadedAudioBufReader = std::sync::Arc::new(std::sync::Mutex::new(_audio_input_buff_reader));

    #[cfg(feature = "audio_pass_test")]
    {
        audio_output_buff_reader = audio_input_buff_reader;
    };

    let mut receive_pipeline= rustxxx::rx_pipeline::RxPipeline::new(
        &FT8, 
        runtime,
    );

    let _audio_input_stream = 
    //     if let Some(audio_input_file_name) = &opt.input_file {
    //     do_audio_file_input(*runtime, &mut audio_input_buff_writer, audio_input_file_name, &mut audio_input_from_channels, &mut audio_input_from_rate)?
    // } else 
    if let Some(audio_input_device_name) = &input_device {
        dbg!(&audio_input_device_name);

        // let input_buff = circular_buffer::CircularBuffer::<{constant::INPUT_BUFSIZE}, f32>::boxed();
        // eg cargo run -- --input-device 'coreaudio:AppleUSBAudioEngine:ZOOM Corporation:UAC-232:2100000:1,2'

        let (audio_input_device, audio_input_config) = cpal_helper::get_audio_input_device_default_config(&host, &audio_input_device_name)?;
        dbg!(&audio_input_config);

        // Will be running the input stream on a separate thread.
        // let stream_receiver = Some(stream_receiver);
        // let receive_pipeline = std::sync::Arc::new(std::sync::Mutex::new(Some(receive_pipeline)));
        // let receive_pipeline = receive_pipeline.clone();

        audio_input_from_channels = audio_input_config.channels() as usize;
        audio_input_from_rate = audio_input_config.sample_rate();

        let audio_input_to_channels = 1;
        let audio_input_to_rate = runtime.target_input_sample_rate().0 as u32;

        dbg!(audio_input_from_channels, audio_input_to_channels, audio_input_from_rate, audio_input_to_rate);

        fn audio_input_data_callback(
            input: &[f32], 
            writer: &mut AudioBufWriter,
        ) {
            // if let Ok(mut guard) = writer.try_lock() 
            {
                // let writer = guard.as_mut();
                // dbg!("WOOHOO: Summink to write");
                for sample in input.iter() {
                    // dbg!(*sample);
                    match writer.try_push(*sample) {
                        Ok(()) => {},
                        Err(_) => {
                            panic!("input_buf overrun - discarding samples");
                        }
                    }
                }
            }
        }

        // let input_buf_writer = Arc::new(Mutex::new(Some(input_buff_writer)));

        // this ignores buf and sets to Default, and strips SampleFormat
        let audio_input_config: StreamConfig = audio_input_config.into();  
        
        // this will spawn a system thread that runs the callback
        // callback should be lightweight
        // NB when ownership of 'stream' is lost then it is shutdown!!
        let audio_input_stream = audio_input_device.build_input_stream(
            &audio_input_config,
            move |data, _: &_| audio_input_data_callback(
                data, 
                &mut audio_input_buff_writer,
            ),
            audio_err_callback,
            None, 
        ).expect("Cannot create audio input stream");

        audio_input_stream.play().expect("Cannot play audio input stream");

        Some(audio_input_stream)
    } else {
        None
    };

    dbg!();

    // #[cfg(not(feature = "audio_pass_test"))] 
    {
        // use proto_ft8::protocol::FT8;

        // could not init this until know the input stream info
        let mut resample_context = receive_pipeline.resample_context(
            audio_input_from_channels, audio_input_from_rate, 
        );

        let ft8_context = ft8_message::FT8Context::new();
        
        let mut audio_buff = [0f32; rx_pipeline::RX_IN_BUFLEN];
        let from_channels = resample_context.from_channels;
        let audio_read_size = rx_pipeline::RX_IN_BUFLEN * from_channels;

        // this will be our main event loop
        while receive_pipeline.continue_run() {
            use ringbuf::traits::Observer;

            if audio_input_buff_reader.occupied_len() >= audio_read_size {
                {
                    // unload the ringbuf into a sized Vec to pass to rustxxx
                    let audio_iter = audio_input_buff_reader.pop_iter();
                    let mut n = 0;
                    for sample in audio_iter.step_by(from_channels).take(audio_read_size) {
                        audio_buff[n] = sample;
                        n += 1;
                    }
                }
                let messages = receive_pipeline.write_mono_sample_buffer(
                    &audio_buff,
                    &mut resample_context
                )?;

                for msg in messages {
                    let cw = &msg.codeword().0;
                    let cw: [u8; ft8_message::FT8_PAYLOAD_BYTES] = cw[..ft8_message::FT8_PAYLOAD_BYTES].try_into()?;
                    let msg = ft8_context.ft8_payload_to_message(&cw)?;
                    dbg!(&msg);
                }
                receive_pipeline.update_spectrogram();
            }
        }   
    }

    dbg!{"RECEIVE DONE"};

    #[cfg(feature = "audio_pass_test")]
    {
        dbg!{"RUNNING AUDIO PASS THROUGH"};
        loop {};
    }

    match _audio_input_stream {
        Some(_stream) => {
        },
        None => {}
    }

    Ok(())
}


fn main() -> Result<(), anyhow::Error> {
    color_backtrace::install();
    let opt = Opt::parse();

    // let loop_back = opt.loop_back.unwrap();
    let runtime: &'static rustxxx::types::Runtime = &FT8_RUNTIME;

    // println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    // let available_hosts = cpal::available_hosts();
    // println!("Available hosts:\n  {available_hosts:?}");

    // for host_id in available_hosts {
    //     println!("{}", host_id.name());
    // }

    #[cfg(any(feature = "enable_tx", test))]
    let tx_thread_handle = std::thread::spawn(move || { tx_main(&opt.output_device, &runtime) });

    // keep receiver in main thread so it can use its debug window
    #[cfg(any(feature = "enable_rx", test))]
    let _ = rx_main(&opt.input_device, &runtime);
    
    let _ = tx_thread_handle.join();

    Ok(())
}

