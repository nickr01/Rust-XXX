// needed for Traits
use clap::Parser;

use cpal::{
    StreamConfig, 
    // SupportedStreamConfig
};
// use cpal::{Sample, SupportedStreamConfig};
use cpal::traits::{
    HostTrait, 
    DeviceTrait, 
    StreamTrait
};

pub fn get_audio_input_device_by_id(host: &cpal::Host, audio_input_device_id: &String) -> Result<cpal::Device, anyhow::Error> {
    let audio_input_device_id: &cpal::DeviceId = &audio_input_device_id.parse()?;
    dbg!(audio_input_device_id);
    match host.device_by_id(audio_input_device_id) {
        Some(device) => Ok(device),
        None => { return Err(anyhow::anyhow!("Cannot get input device by id {}", audio_input_device_id)) }
    }
}

pub fn get_audio_input_device_by_name(host: &cpal::Host, audio_input_device_name: &String) -> Result<cpal::Device, anyhow::Error> {
    dbg!(audio_input_device_name);
    for device in host.input_devices()? {
        let desc =  device.description()?;
        let desc = desc.name();
        if desc == audio_input_device_name {
            return Ok(device);
        }
    }
    Err(anyhow::anyhow!("Cannot get input device by name {}", audio_input_device_name))
}
        
pub fn get_audio_input_device(host: &cpal::Host, audio_input_device_name: &String) -> Result<cpal::Device, anyhow::Error> {
    if audio_input_device_name.is_empty() {
        match host.default_input_device() {
            Some(device) => Ok(device),
            None => { return Err(anyhow::anyhow!("Cannot get default input device")) }
        }
    } else {
        match get_audio_input_device_by_name(host, audio_input_device_name) {
            Ok(device) => { Ok(device) },
            Err(_) => {
                get_audio_input_device_by_id(host, audio_input_device_name)
            }
        }
    }
}

pub fn get_audio_input_device_default_config(host: &cpal::Host, audio_input_device_name: &String,) -> 
    Result<(cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let audio_input_device = get_audio_input_device(host, audio_input_device_name)?;
    if audio_input_device.supports_input() {
        dbg!();
        let config = audio_input_device.default_input_config()?;
        Ok((audio_input_device, config))
    } else {
        dbg!();
        Err(anyhow::anyhow!("Input device does not support input {}", audio_input_device_name))
    }
}

pub fn get_audio_output_device_by_id(host: &cpal::Host, audio_output_device_id: &String) -> Result<cpal::Device, anyhow::Error> {
    let audio_output_device_id: &cpal::DeviceId = &audio_output_device_id.parse()?;
    dbg!(audio_output_device_id);

    match host.device_by_id(audio_output_device_id) {
        Some(device) => Ok(device),
        None => { return Err(anyhow::anyhow!("Cannot get output device by id {}", audio_output_device_id)) }
    }
}

pub fn get_audio_output_device_by_name(host: &cpal::Host, audio_output_device_name: &String) -> Result<cpal::Device, anyhow::Error> {
    dbg!(audio_output_device_name);

    for device in host.output_devices()? {
        let desc =  device.description()?;
        let desc = desc.name();
        if desc == audio_output_device_name {
            return Ok(device);
        }
    }
    Err(anyhow::anyhow!("Cannot get output device by name {}", audio_output_device_name))
}
        
pub fn get_audio_output_device(host: &cpal::Host, audio_output_device_name: &String) -> Result<cpal::Device, anyhow::Error> {
    if audio_output_device_name.is_empty() {
        match host.default_output_device() {
            Some(device) => Ok(device),
            None => { return Err(anyhow::anyhow!("Cannot get default output device")) }
        }
    } else {
        match get_audio_output_device_by_name(host, audio_output_device_name) {
            Ok(device) => { Ok(device) },
            Err(_) => {
                get_audio_output_device_by_id(host, audio_output_device_name)
            }
        }
    }
}

pub fn get_audio_output_device_default_config(host: &cpal::Host, audio_output_device_name: &String,) -> 
    Result<(cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
    let audio_output_device = get_audio_output_device(host, audio_output_device_name)?;
    if audio_output_device.supports_output() {
        dbg!();
        let config = audio_output_device.default_output_config()?;
        Ok((audio_output_device, config))
    } else {
        dbg!();
        Err(anyhow::anyhow!("output device does not support output {}", audio_output_device_name))
    }
}

// fn get_audio_output_device(host: &cpal::Host, audio_output_device_name: &String) -> Result<cpal::Device, anyhow::Error> {
//     if audio_output_device_name.is_empty() {
//         match host.default_output_device() {
//             Some(device) => Ok(device),
//             None => { return Err(anyhow::anyhow!("Cannot get default output device")) }
//         }
//     } else {
//         let audio_output_device_id: &cpal::DeviceId = &audio_output_device_name.parse()?;
//         dbg!(audio_output_device_id);

//         match host.device_by_id(audio_output_device_id) {
//             Some(device) => Ok(device),
//             None => { return Err(anyhow::anyhow!("Cannot get output device by id {}", audio_output_device_name)) }
//         }
//     }
// }

// fn get_audio_output_device_default_config(host: &cpal::Host, audio_output_device_name: &String,) -> 
//     Result<(cpal::Device, cpal::SupportedStreamConfig), anyhow::Error> {
//     let audio_output_device = get_audio_output_device(host, audio_output_device_name)?;
//     if audio_output_device.supports_output() {
//         dbg!();
//         let config = audio_output_device.default_output_config()?;
//         Ok((audio_output_device, config))
//     } else {
//         dbg!();
//         Err(anyhow::anyhow!("Output device does not support output {}", audio_output_device_name))
//     }
// }

// fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
//     if format.is_float() {
//         hound::SampleFormat::Float
//     } else {
//         hound::SampleFormat::Int
//     }
// }

// fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
//     hound::WavSpec {
//         channels: config.channels() as _,
//         sample_rate: config.sample_rate() as _,
//         bits_per_sample: (config.sample_format().sample_size() * 8) as _,
//         sample_format: sample_format(config.sample_format()),
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    // Https://docs.rs/jack/latest/jack/

    #[test]
    fn test_audio_input_device_by_name() {
        let host = cpal::default_host();
        let name: String = "Loopback Audio".to_string();
        let _ = get_audio_input_device_by_name(&host, &name)
            .expect("Cannot get device by name {}");
    }

    #[test]
    fn test_audio_input_device_by_id() {
        let host = cpal::default_host();
        let id: String = "coreaudio:com.rogueamoeba.Loopback:FDC858DA-EA9D-469B-9B86-2C4ADC20537E".to_string();
        let _ = get_audio_input_device_by_id(&host, &id)
            .expect("Cannot get device by name");
    }

    #[test]
    fn test_audio_input_config_by_name() {
        let host = cpal::default_host();
        let name: String = "Loopback Audio".to_string();
        let _ = get_audio_input_device_by_name(&host, &name)
            .expect("Cannot get device by name");
    }

    #[test]
    fn test_audio_input_device_default_config_by_name() {
        let host = cpal::default_host();
        let name: String = "Loopback Audio".to_string();
        let (_, _) = get_audio_input_device_default_config(&host, &name)
            .expect("Cannot get device and config");
    }

    #[test]
    fn test_audio_input_device_default_config_by_id() {
        let host = cpal::default_host();
        let id: String = "coreaudio:com.rogueamoeba.Loopback:FDC858DA-EA9D-469B-9B86-2C4ADC20537E".to_string();
        let (_, _) = get_audio_input_device_default_config(&host, &id)
            .expect("Cannot get device and config");
    }

    #[test]
    fn test_audio_output_device_by_name() {
        let host = cpal::default_host();
        let name: String = "MacBook Pro Speakers".to_string();
        let _ = get_audio_output_device_by_name(&host, &name)
            .expect("Cannot get device by name");
    }

    #[test]
    fn test_audio_output_device_by_id() {
        let host = cpal::default_host();
        let id: String = "coreaudio:BuiltInSpeakerDevice".to_string();
        let _ = get_audio_output_device_by_id(&host, &id)
            .expect("Cannot get device by id");
    }

    #[test]
    fn test_audio_output_device_default_config_by_name() {
        let host = cpal::default_host();
        let id: String = "MacBook Pro Speakers".to_string();
        let (_, _) = get_audio_output_device_default_config(&host, &id)
            .expect("Cannot get device and config");
    }

    #[test]
    fn test_audio_output_device_default_config_by_id() {
        let host = cpal::default_host();
        let id: String = "coreaudio:BuiltInSpeakerDevice".to_string();
        let (_, _) = get_audio_output_device_default_config(&host, &id)
            .expect("Cannot get device and config");
    }
}