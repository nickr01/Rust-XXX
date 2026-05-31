// src/FT8_stream.rs

// devices: i/o use clap
// files: i/o use wav_io - but note compat issue with some rodio headers
// (move js8record to use wav_io?)
// why wav_io and not hound? - more recent and supports resampling

// use std::sync::mpsc::Receiver;

use anyhow::{Context, Result}; // - for user level

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

// use crate::constant::{INPUT_BUFSIZE, InputBufWriter};
// use crate::rustxxx::InputBufReader;
// use crate::rustxxx::AudioSampleBuffer;
use transport::rustxxx::{
    AudioSampleBuffer, 
    // Runtime
};
// use crate::rustxxx::InputBufWriter;

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
mod cpal_helper;
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

        
#[derive(clap::Parser, std::fmt::Debug)]
#[command(version, about = "Rust-XXX FT8-like modem testbed", long_about = None)]
struct Opt {
    /// The audio input device to use.
    #[arg(short, long, default_value = "")]
    input_device: Option<String>,

    /// The audio input file to use. 
    #[arg(long)]
    input_file: Option<String>,

    /// The audio output device to use.
    #[arg(short, long, default_value = "")]
    output_device: Option<String>,

    /// The audio input file to use. 
    #[arg(long)]
    output_file: Option<String>,

    #[arg(short, long, default_value = "true")]
    loop_back: Option<bool>,

    //  How long to record, in seconds
    // #[arg(long, default_value_t = 15)]
    // duration: u64,

    // Slowest js8speed in test. Determines time modulus.
    // #[arg(short, long, default_value = "normal")]
    // speed: Speed,
}

#[cfg(any(feature = "enable_rx", test))]
fn do_audio_file_input(
    runtime: transport::rustxxx::Runtime, 
    input_buff_writer: &mut transport::rustxxx::AudioBufWriter, 
    input_file: String,
    from_channels: &mut usize,
    from_rate: &mut u32
) -> Result<Option<cpal::Stream>, anyhow::Error>
{
    // const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/", input_file, ".wav");
    // let spec = wav_spec_from_config(&config);
    // let writer = hound::WavWriter::create(PATH, spec)?;
    // let writer = std::sync::Arc::new(std::sync::Mutex::new(Some(writer)));

    // Input from file
    let input_wav = std::fs::File::open(&input_file)
        .context(format!("Cannot open input wav file {}", input_file))?;

    let (header, signal) = wav_io::read_from_file(input_wav)
        .context(format!("Cannot read from wav file {}", input_file))?;

    dbg!(&header);

    *from_channels = header.channels as usize;
    *from_rate = header.sample_rate as u32;

    // if header.channels != runtime.channels().0 as u16 {
    //     let new_channels = runtime.channels().0 as u16;
    //     dbg!(new_channels);
    //     signal = wav_io::utils::stereo_to_mono(signal);
    //     header.channels = new_channels;
    // }

    // dbg!(runtime.target_input_sample_rate());
    // {
    //     let target_sample_rate = runtime.target_input_sample_rate().0 as u32;
    //     if header.sample_rate != target_sample_rate {
    //         dbg!(header.sample_rate, target_sample_rate);
    //         signal = wav_io::resample::linear(
    //             signal, 
    //             runtime.channels().0 as u16, 
    //             header.sample_rate, 
    //             target_sample_rate
    //         );
    //         header.sample_rate = target_sample_rate;
    //     }
    // }

    // dbg!(runtime.subtracts());

    // let mut file_out = File::create("./out/resampled.wav").expect();
    // writer::to_file(&mut file_out, &WavData::new(header, samples.clone())).expect();

    dbg!(signal.len());

    dbg!(
        runtime.rx_symbol_osr(),
        runtime.rx_freq_osr()
    );

    // let input_buf = ringbuf::HeapRb::<f32>::new(signal.len());
    // for testing we'll preload a buffer block
    // if let Ok(mut guard) = input_buff_writer.try_lock() 
    {
        // let input_buff_writer = guard.as_mut();
        for sample in signal.iter() {
            input_buff_writer.try_push(*sample).expect("input_buf overrun");
        }
    }
    
    Ok(None)
}

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

fn main() -> Result<(), anyhow::Error> {
    #[cfg(any(feature = "enable_tx", test))]
    tx_main()?;

    #[cfg(any(feature = "enable_rx", test))]
    rx_main()?;
    
    Ok(())
}

#[cfg(any(feature = "enable_tx", test))]
fn tx_main() -> Result<(), anyhow::Error> {
    Ok(())
}

#[cfg(any(feature = "enable_rx", test))]
fn rx_main() -> Result<(), anyhow::Error> {
    color_backtrace::install();
    
    let opt = Opt::parse();

    // let loop_back = opt.loop_back.unwrap();

    let runtime: &'static transport::rustxxx::Runtime = &transport::rustxxx::TEST_FT8_RUNTIME;

    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let available_hosts = cpal::available_hosts();
    println!("Available hosts:\n  {available_hosts:?}");

    for host_id in available_hosts {
        println!("{}", host_id.name());
    }

    // Set up the CPAL input device and stream with the default input config.
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
        eprintln!("an error occurred on audio stream: {err}");
    };

    // these get init by the device init blocks
    // pipeline needs to know these as conversions happen in the pipeline i/o
    // which leaves the audio thread callbacks as light as possible
    let mut audio_input_from_channels = 0; 
    let mut audio_input_from_rate = 0;
    let mut _audio_output_to_channels = 0; 
    let mut _audio_output_to_rate = 0;

    let audio_input_buffer: AudioSampleBuffer = ringbuf::HeapRb::<f32>::new(transport::rustxxx::AUDIO_INPUT_BUFSIZE);
    let audio_output_buffer: AudioSampleBuffer = ringbuf::HeapRb::<f32>::new(transport::rustxxx::AUDIO_OUTPUT_BUFSIZE);

    let (mut audio_input_buff_writer, mut audio_input_buff_reader) = audio_input_buffer.split();
    // let mut audio_input_buff_writer: transport::rustxxx::ThreadedAudioBufWriter = std::sync::Arc::new(std::sync::Mutex::new(_audio_input_buff_writer));
    // let mut audio_input_buff_reader: transport::rustxxx::ThreadedAudioBufReader = std::sync::Arc::new(std::sync::Mutex::new(_audio_input_buff_reader));

    let (mut _audio_output_buff_writer, mut audio_output_buff_reader) = audio_output_buffer.split();
    // let mut _audio_output_buff_writer: transport::rustxxx::ThreadedAudioBufWriter = std::sync::Arc::new(std::sync::Mutex::new(_audio_output_buff_writer));
    // let mut audio_output_buff_reader: transport::rustxxx::ThreadedAudioBufReader = std::sync::Arc::new(std::sync::Mutex::new(_audio_output_buff_reader));

    #[cfg(feature = "audio_pass_test")]
    {
        audio_output_buff_reader = audio_input_buff_reader;
    };

    let mut receive_pipeline= transport::pipeline::Pipeline::new(
        &proto_ft8::protocol::FT8, 
        runtime,
    );

    let _audio_input_stream = if let Some(audio_input_file_name) = opt.input_file {
        do_audio_file_input(*runtime, &mut audio_input_buff_writer, audio_input_file_name, &mut audio_input_from_channels, &mut audio_input_from_rate)?
    } else if let Some(audio_input_device_name) = opt.input_device {
        dbg!(&audio_input_device_name);

        // let input_buff = circular_buffer::CircularBuffer::<{constant::INPUT_BUFSIZE}, f32>::boxed();
        // eg cargo run -- --input-device 'coreaudio:AppleUSBAudioEngine:ZOOM Corporation:UAC-232:2100000:1,2'

        let (audio_input_device, audio_input_config) = crate::cpal_helper::get_audio_input_device_default_config(&host, &audio_input_device_name)?;
        dbg!(&audio_input_config);

        // Will be running the input stream on a separate thread.
        // let stream_receiver = Some(stream_receiver);
        // let receive_pipeline = std::sync::Arc::new(std::sync::Mutex::new(Some(receive_pipeline)));
        // let receive_pipeline = receive_pipeline.clone();

        audio_input_from_channels = audio_input_config.channels() as usize;
        audio_input_from_rate = audio_input_config.sample_rate();

        let audio_input_to_channels = runtime.channels().0;
        let audio_input_to_rate = runtime.target_input_sample_rate().0 as u32;

        dbg!(audio_input_from_channels, audio_input_to_channels, audio_input_from_rate, audio_input_to_rate);

        fn audio_input_data_callback(
            input: &[f32], 
            writer: &mut transport::rustxxx::AudioBufWriter,
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

    let _audio_output_stream = if let Some(_output_audio_file_name) = opt.output_file {
        todo!();
        // do_file_output(*runtime, &mut input_buff_writer, input_file_name, &mut from_channels, &mut from_rate)?
        // None
    } else if let Some(audio_output_device_name) = opt.output_device {
        dbg!(&audio_output_device_name);

        let (audio_output_device, audio_output_config) = cpal_helper::get_audio_output_device_default_config(&host, &audio_output_device_name)?;
        dbg!(&audio_output_config);

        let audio_output_from_channels = runtime.channels().0;
        let audio_output_from_rate = runtime.target_input_sample_rate().0 as u32;

        _audio_output_to_channels = audio_output_config.channels() as usize;
        _audio_output_to_rate = audio_output_config.sample_rate();

        dbg!(audio_output_from_channels, _audio_output_to_channels, audio_output_from_rate, _audio_output_to_rate);

        fn audio_output_data_callback(output: &mut [f32], reader: &mut transport::rustxxx::AudioBufReader) {
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

    dbg!();

    // #[cfg(not(feature = "audio_pass_test"))] 
    {
        // use proto_ft8::protocol::FT8;

        // could not init this until know the input info
        let mut resample_context = receive_pipeline.resample_context(
            audio_input_from_channels, 
            audio_input_from_rate, 
        );

        // this will be our main event loop
        while receive_pipeline.continue_run() {
            let codewords = receive_pipeline.write_sample_buffer(
                &mut audio_input_buff_reader,
                &mut resample_context
            )
                .context("Cannot run the receiver").unwrap();

            receive_pipeline.update_spectrogram();

            let mut ft8_messages: Vec<String> = Vec::new();
            for codeword in codewords {
                match proto_ft8::unpack_ft8::unpack77(&codeword) {
                    Some(msg) => {
                        dbg!(&msg);
                        ft8_messages.push(msg);
                    },
                    None => {
                        dbg!("Bad unpack");
                    }
                }
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

    match _audio_output_stream {
        Some(_stream) => {
        },
        None => {}
    }

    Ok(())
}
