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
    let host = cpal::default_host();
    let device = host.default_output_device().expect("failed to find output device");
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);

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

    let mut engine = Engine::<1>::new();
    engine.set_sr(sample_rate);
    engine.set_bpm(120.0);
    engine.update_with_code("~t: sin 439\no: sin 440 >> add ~t");


    let mut next_value = move || {
        let (block, _err_msg) = engine.next_block(vec![]);
        block[0][0]
    };

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

