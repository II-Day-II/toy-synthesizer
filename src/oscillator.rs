use crate::consts::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Off,
    Freq(f32),
    Amp(bool),
}

pub trait Oscillator {
    fn make_noise(&mut self) -> f32;
    fn handle_message(&mut self, msg: Message);
}
pub struct SinOscillator {
    phi: f32,
    freq: f32,
    amp: f32,
    sample_rate: f32,
    on: bool,
}

impl SinOscillator {
    pub fn new(sample_rate: f32) -> Self {
        SinOscillator {
            phi: 0.0,
            freq: BASE_FREQ,
            amp: 1.0,
            on: false,
            sample_rate,
        }
    }
}

impl Oscillator for SinOscillator {
    fn make_noise(&mut self) -> f32 {
        self.phi = (self.phi + self.freq / self.sample_rate).fract();
        self.on as i32 as f32 * self.amp * 0.1 * (PI * 2.0 * self.phi).sin()
    }
    fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Off => self.on = false,
            Message::Amp(v) => {
                self.amp += if v {0.1} else {-0.1};
            },
            Message::Freq(v) => {
                self.freq = v;
                self.on = true;
            },
        }
    }
}