extern crate anyhow;
extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config()?;

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into())?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into())?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into())?,
    }

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    //let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let vector: Arc<Mutex<VecDeque<f32>>> = Arc::new(Mutex::new(VecDeque::with_capacity(1000000)));

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let vec_cloned = vector.clone();
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &vec_cloned)
        },
        err_fn,
    )?;
    println!("go!");
    stream.play()?;
    if let Ok(mut queue) = vector.lock() {
        fill_queue(&mut queue, 2048);
    }

    for _t in 0..1000 {
        std::thread::sleep(std::time::Duration::from_millis(1));
        if let Ok(mut queue) = vector.lock() {
            while queue.len() < 2048 {
                //println!("filling, len is {}", queue.len());
                fill_queue(&mut queue, 1024);
            }
        }
    }
    Ok(())
}

fn fill_queue(sharedqueue: &mut VecDeque<f32>, n: usize) {
    for n in 0..n {
        let val = (n as f32 / 1024.0 * 10.0 * 2.0 * 3.141592).sin()*0.25;
        sharedqueue.push_back(cpal::Sample::from::<f32>(&val));
    }
} 

fn write_data<T>(output: &mut [T], channels: usize, sharedqueue: &Arc<Mutex<VecDeque<f32>>>)
where
    T: cpal::Sample,
{
    //println!("New buffer, length {}", output.len());
    if let Ok(mut queue) = sharedqueue.lock() {
        let skip = if output.len() > queue.len()*channels {
            output.len()/channels - queue.len()
        }
        else {
            0
        };
        for frame in output.chunks_mut(channels).skip(skip) {
            let value: T = cpal::Sample::from::<f32>(&queue.pop_front().unwrap_or(0.0));
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
}
