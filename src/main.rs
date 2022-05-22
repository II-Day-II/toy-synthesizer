// David Kaméus
// 22/05/2022

use tuix::{*, widgets::*};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::thread;

static THEME: &'static str = include_str!("theme.css");

#[derive(Clone, Copy, Debug, PartialEq)]
enum Message {
    Note(bool),
    Freq(f32),
    Amp(f32),
}

struct Controller {
    command_sender: crossbeam_channel::Sender<Message>,
    amp_knob: Entity,
    freq_knob: Entity,
}

impl Controller {
    pub fn new(command_sender: crossbeam_channel::Sender<Message>) -> Self {
        Controller {
            command_sender,
            amp_knob: Entity::null(),
            freq_knob: Entity::null()
        }
    }
}

impl Widget for Controller {
    type Ret = Entity;
    type Data = ();
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret { // set this widget to focused when created so events go through it
        let row = Row::new().build(state, entity, |builder| {
            builder
                .set_child_space(Stretch(1.0))
                .set_col_between(Stretch(1.0))
        });
        let map = GenericMap::new(0.0, 1.0, ValueScaling::Linear, DisplayDecimals::One, None);
        self.amp_knob = Knob::new(map, 1.0).build(state, row, |builder| {
            builder.set_width(Pixels(50.0))
        });
        let map = FrequencyMap::new(440.0, 2000.0, ValueScaling::Linear, FrequencyDisplayMode::default(), true);

        self.freq_knob = Knob::new(map, 0.0).build(state, row, |builder| {
            builder
                .set_width(Pixels(50.0))
        });
        state.focused = entity;
        entity
    }
    fn on_event(&mut self, _state: &mut State, _entity: Entity, event: &mut Event) {
        if let Some(window_event) = event.message.downcast::<WindowEvent>() {
            match window_event {
                WindowEvent::KeyDown(Code::KeyZ, _) => {
                    self.command_sender.send(Message::Note(true)).expect("Failed to send Note on message!")
                }
                WindowEvent::KeyUp(Code::KeyZ, _) => {
                    self.command_sender.send(Message::Note(false)).expect("Failed to send Note off message!")
                }
                _ => {}
            }
        }
        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                SliderEvent::ValueChanged(val) => {
                    if event.target == self.amp_knob {
                        self.command_sender.send(Message::Amp(*val)).expect("Error sending amplitute change!");
                    }
                    if event.target == self.freq_knob {
                        self.command_sender.send(Message::Freq(*val)).expect("Error sending frequency change!");
                    }
                }
                _=> {}
            }
        }
    }
}

fn main() {
    let (tx, rx) = crossbeam_channel::bounded(1024);

    // spawn audio thread
    thread::spawn(move || {
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

    // create GUI
    let window_description = WindowDescription::new().with_title("Toy synthesizer").with_inner_size(200, 120);
    let app = Application::new(window_description, |state, window| {
        state.style.parse_theme(THEME);
        window.set_background_color(state, Color::rgb(60, 60, 60));
        Controller::new(tx.clone()).build(state, window, |builder| builder);
    });

    app.run();
}

fn run<T>(device: &cpal::Device, cfg: &cpal::StreamConfig, rx: crossbeam_channel::Receiver<Message>) -> anyhow::Result<()> where T: cpal::Sample {
    // get config info to reasonable names and types
    let sample_rate = cfg.sample_rate.0 as f32;
    let channels = cfg.channels as usize;
    // error callback for output stream
    let err_fn = |err| eprintln!("Error occured on stream: {}", err);

    // oscillator variables
    let mut phi = 0.0;
    let mut freq = 440.0;
    let mut amp = 1.0 ;
    let mut note = false;

    // Build output stream
    let stream = device.build_output_stream(
        cfg,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                while let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        Message::Note(v) => note = v,
                        Message::Amp(v) => amp = v,
                        Message::Freq(v) => freq = v * (2000.0 - 440.0) + 440.0,
                    }
                }

                // 'phase clock' varying between 0.0 and 1.0 with rate of freq
                phi = (phi + freq/sample_rate).fract();
                let make_noise = |phi: f32| -> f32 {note as i32 as f32 * amp * 0.1 * (2.0 * 3.141592 * phi).sin()};
                // convert to Sample
                let value: T = cpal::Sample::from(&make_noise(phi));

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