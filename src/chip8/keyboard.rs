use std::collections::HashMap;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Keyboard {
    pressed_key: HashMap<u8, u8>,
    last_pressed_key: Option<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Keyboard {
            pressed_key: HashMap::new(),
            last_pressed_key: None,
        }
    }

    fn keycode_to_hex(keycode: &Keycode) -> Option<u8> {
        let hex =
            match keycode {
                Keycode::Num1 => 0x01,
                Keycode::Num2 => 0x02,
                Keycode::Num3 => 0x03,
                Keycode::Num4 => 0x0C,
                Keycode::Q => 0x04,
                Keycode::W => 0x05,
                Keycode::E => 0x06,
                Keycode::R => 0x0D,
                Keycode::A => 0x07,
                Keycode::S => 0x08,
                Keycode::D => 0x09,
                Keycode::F => 0x0E,
                Keycode::Z => 0x0A,
                Keycode::X => 0x00,
                Keycode::C => 0x0B,
                Keycode::V => 0x0F,
                _ => 0xFF
            } as u8;
        if hex == 0xFF {
            None
        } else {
            Some(hex)
        }
    }
    pub fn on_keyboard_event(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(ref key), .. } => {
                #[cfg(feature = "log_debug")]
                println!("KeyDown => {}", key);
                if let Some(hex) = Keyboard::keycode_to_hex(key) {
                    self.pressed_key.insert(hex, 1);
                    self.last_pressed_key = Some(hex);
                }
            }
            Event::KeyUp { keycode: Some(ref key), .. } => {
                #[cfg(feature = "log_debug")]
                println!("KeyUp => {}", key);
                if let Some(hex) = Keyboard::keycode_to_hex(key) {
                    self.pressed_key.insert(hex, 0);
                }
            }
            _ => {}
        }
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        let v = self.pressed_key.get(&key)
            .map(|v| *v).unwrap_or(0);
        v == 1
    }

    pub fn last_pressed_key(&mut self) -> Option<u8> {
        self.last_pressed_key.take()
    }
}