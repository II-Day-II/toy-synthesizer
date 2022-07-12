use crate::consts::*;
use badrand;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Message {
    Off,
    Freq(f32),
    Amp(bool),
}

pub enum OscType {
    Sin,
    Sqr,
    Tri,
    Saw(bool),
    Rnd,
}

pub struct Oscillator {
    phi: f32,
    freq: f32,
    amp: f32,
    sample_rate: f32,
    on: bool,
    rng: badrand::Rng
}

impl Oscillator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            phi: 0.0,
            freq: BASE_FREQ,
            amp: 1.0,
            on: false,
            sample_rate,
            rng: badrand::Rng::new(),
        }
    }
    pub fn tick(&mut self) { // update phase clock
        self.phi = (self.phi + self.freq / self.sample_rate).fract();
    }
    pub fn make_noise(&mut self, osc: OscType) -> f32 {
        self.on as i32 as f32 * self.amp * 0.1 * match osc {
            OscType::Sin => {
                (PI * 2.0 * self.phi).sin()
            },
            OscType::Sqr => {
                0.5 * if (PI * 2.0 * self.phi).sin() > 0.0 {1.0} else {-1.0} 
            },
            OscType::Tri => {
                (PI * 2.0 * self.phi).sin().asin() * 2.0 / PI
            },
            OscType::Saw(fast) => {
                if fast {
                    (2.0 / PI) * ((self.phi % 1.0) - PI / 2.0)//(dHertz * PI * fmod(dTime, 1.0 / dHertz) - (PI / 2.0));
                }
                else {
                    let mut o = 0.0;
                    for n in (1..100).map(|x| x as f32) {
                        o += (PI * 2.0 * self.phi * n).sin() / n;
                    }
                    o * (2.0 / PI) * 0.5
                }
            },
            OscType::Rnd => {
                self.rng.range_f32(-0.1..0.1)
            }

        }
    }
    pub fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Off => self.on = false,
            Message::Amp(v) => {
                self.amp += if v {0.1} else {-0.1};
                eprintln!("{}", self.amp);
            },
            Message::Freq(v) => {
                self.freq = v;
                self.on = true;
            },
        }
    }
}