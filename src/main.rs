use std::alloc::System;

#[global_allocator]
static A: System = System;

use std::error::Error;
use std::time::Duration;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SizedSample,
};
use glicol::Engine;

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .and_then(|a| cpal::host_from_id(a).ok())
            .or(Some(cpal::default_host()))
            .ok_or("Couldn't find audio host")?;
    let device = host.default_output_device().ok_or("failed to find output device")?;
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);

    /* ALSA setup
    // We have to use one channel or else we hit https://github.com/RustAudio/cpal/issues/479
    let config = device.supported_output_configs()?.find(|c| {
        c.channels() == 1 && c.sample_format() == cpal::SampleFormat::F32 
    }).ok_or("couldn't find supported config")?.with_sample_rate(cpal::SampleRate(44100));
    println!("Output config: {:?}", config);
    */

    match config.sample_format() {
        cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        // cpal::SampleFormat::I24 => run::<I24>(&device, &config.into()),
        cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
        // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
        cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
        cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
        // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
        cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
        sample_format => panic!("Unsupported sample format '{sample_format}'"),
    }
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), Box<dyn Error>>
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut engine = Engine::<1>::new();
    engine.set_sr(sample_rate);
    engine.set_bpm(120.0);
    engine.update_with_code("~t: sin 439\no: sin 440 >> add ~t >> mul 0.1");

    let mut block_pos = 0;
    let (mut block, _err_msg) = engine.next_block(vec![]);
    let mut next_value = move || {
        block = engine.next_block(vec![]).0;
        block[0][0]
    };

    /* Sin test
    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };
    */

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    fn write_data<T : Sample + FromSample<f32>>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
    {
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(next_sample());
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_millis(8));
    }
}

