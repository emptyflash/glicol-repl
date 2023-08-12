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
use std::sync::{Mutex, Arc};

const BLOCK_SIZE: usize = 128;

fn main() -> Result<(), Box<dyn Error>> {
    let host = cpal::default_host();
    /* handle jack
    let host = cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .and_then(|a| cpal::host_from_id(a).ok())
            .or(Some(cpal::default_host()))
            .ok_or("Couldn't find audio host")?;
    */
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
    let sample_rate = config.sample_rate.0 as usize;
    let channels = config.channels as usize;

    let engine_mutex = Arc::new(Mutex::from(Engine::<BLOCK_SIZE>::new()));
    let engine_mutex_inner = Arc::clone(&engine_mutex);
    let mut engine = engine_mutex.lock().unwrap();
    engine.set_sr(sample_rate);
    engine.set_bpm(120.0);
    engine.update_with_code("~t: sin 439\no: sin 440 >> add ~t >> mul 0.1");
    drop(engine);

    /* Sin test
    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };
    */

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            let mut engine_inner = engine_mutex_inner.lock().unwrap();
            let mut block_pos = 0;
            let mut block = engine_inner.next_block(vec![]).0;
            for frame in data.chunks_mut(channels) {
                let mut channel = 0;
                for sample in frame.iter_mut() {
                    let block_val = block[channel][block_pos];
                    let value: T = T::from_sample(block_val);
                    *sample = value;
                    channel += 1;
                }
                block_pos += 1;
                if block_pos >= BLOCK_SIZE {
                    block = engine_inner.next_block(vec![]).0;
                    block_pos = 0;
                }
            }
        },
        err_fn,
        None,
    )?;
    stream.play()?;

    loop {
        std::thread::sleep(Duration::from_millis(8));
    }
}

