use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;

pub struct Sound {
    device: AudioDevice<SquareWave>,
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

impl Sound {
    pub fn new(sdl_ctx: &Sdl) -> Self {
        let audio_subsystem = sdl_ctx.audio().unwrap();
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let device = audio_subsystem.open_playback(
            None,
            &spec,
            |sepc| {
                SquareWave {
                    phase_inc: 440.0 / spec.freq.unwrap() as f32,
                    phase: 0.0,
                    volume: 0.25,
                }
            },
        )
            .unwrap();
        Sound { device }
    }

    pub fn pause(&self) {
        self.device.pause()
    }

    pub fn resume(&self) {
        self.device.resume()
    }
}