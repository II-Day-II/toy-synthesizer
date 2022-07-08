// David Kaméus
// 22/05/2022

mod linsearch;
mod oscillator;
mod consts;

use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, Sample};
use std::{thread::{self, JoinHandle}};
use crossbeam_channel::{Receiver, Sender};
use device_query::{DeviceState, keymap::Keycode, DeviceQuery,};

use linsearch::Linsearch;
use oscillator::*; 
use consts::*;


fn main() -> anyhow::Result<()> {
    let (tx, rx) = crossbeam_channel::bounded(1024);

    // spawn audio thread
    let handle = thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("failed to find output device!");
        let cfg = device.default_output_config().expect("failed to find device output config!");
        match cfg.sample_format() { // why can this function not just return the type and let me use it directly? >m<
            cpal::SampleFormat::I16 => {
                run::<i16>(&device, &cfg.into(), rx.clone()).expect("error when running i16 format")
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(&device, &cfg.into(), rx.clone()).expect("error when running u16 format")
            }
            cpal::SampleFormat::F32 => {
                run::<f32>(&device, &cfg.into(), rx.clone()).expect("error when running f32 format")
            }
        };
    });

    println!(
"|   |   |   |   |   | |   |   |   |   | |   | |   |   |   | 
|   | S |   |   | F | | G |   |   | J | | K | | L |   |   | Ä
|   |___|   |   |___| |___|   |   |___| |___| |___|   |   |__ 
|     |     |     |     |     |     |     |     |     |     | 
|  Z  |  X  |  C  |  V  |  B  |  N  |  M  |  ,  |  .  |  -  | 
|_____|_____|_____|_____|_____|_____|_____|_____|_____|_____| "
    );

    event_loop(&handle, tx)?;
  
    handle.join().expect("join audio thread"); 
    Ok(())
}

fn event_loop(handle: &JoinHandle<()>, tx: Sender<Message>) -> anyhow::Result<()>{
    let keys: Vec<Keycode> = "ZSXCFVGBNJMK"
        .chars()
        .map(|x| 
            (&x.to_string()).parse().unwrap()
        ).chain(["Comma", "L", "Dot", "Minus", "Apostrophe"]
            .iter()
            .map(|x| 
                x.parse().unwrap()
            )
        ).collect();
        
    let kbm = DeviceState::new();
    let mut key_down;
    let mut key_id = None;
    
    loop {
        key_down = false;
        let pressed_keys = kbm.get_keys();
        if pressed_keys.len() > 0 {
            let p = pressed_keys[0];
            if p == Keycode::Escape {
                handle.thread().unpark();
                break;
            } 
            /* TODO: This does NOT work */
            else if p == Keycode::PageUp {
                tx.send(Message::Amp(true))?;
                key_down = true;
            }
            else if p == Keycode::PageDown {
                tx.send(Message::Amp(false))?;
                key_down = true;
            } 
            else if let Some(k) = keys.search(&p) {
                if key_id != Some(k) {
                    let f = BASE_FREQ * TWELFTH_ROOT_OF_2.powf(k as f32);
                    tx.send(Message::Freq(f))?;
                    key_id = Some(k);
                }
                key_down = true;
            } 
        }
        if !key_down {
            if !key_id.is_none() {
                key_id = None;
            }
            tx.send(Message::Off)?;
        }
    }
    Ok(())
}

fn run<T: Sample>(device: &cpal::Device, cfg: &cpal::StreamConfig, rx: Receiver<Message>) -> anyhow::Result<()> {
    // get config info to reasonable names and types
    let sample_rate = cfg.sample_rate.0 as f32;
    let channels = cfg.channels as usize;
    // error callback for output stream
    let err_fn = |err| eprintln!("Error occured on stream: {}", err);

    let mut osc = SinOscillator::new(sample_rate);

    // Build output stream
    let stream = device.build_output_stream(
        cfg,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                while let Ok(cmd) = rx.try_recv() {
                    osc.handle_message(cmd);
                }

                // convert to Sample
                let value: T = cpal::Sample::from(&osc.make_noise());

                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        err_fn
    )?;

    // play the stream
    stream.play()?;
    // park thread to play sound until app end
    std::thread::park();
    Ok(())
}