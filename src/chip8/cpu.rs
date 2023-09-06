use std::thread;
use std::time::{Duration, Instant};

use rand::Rng;
use sdl2::event::Event;
use sdl2::EventPump;

use crate::chip8::display::Display;
use crate::chip8::keyboard::Keyboard;
use crate::chip8::sound::Sound;

const MEMORY_FONT_START: u16 = 0x0000;
const MEMORY_FONT_END: u16 = 0x01FF;

const MEMORY_PROGRAM_BASE: u16 = 0x0200;
const MEMORY_PROGRAM_END: u16 = 0x0FFF;

const MEMORY_LEN: usize = 0x1000;

pub const MEMORY_PROGRAM_LEN: u16 = (MEMORY_PROGRAM_END - MEMORY_PROGRAM_BASE);

const FPS: u64 = 60;
// 60hz
const MICROS_PER_FRAME: Duration = Duration::from_micros((Duration::from_secs(1).as_micros() as u64) / FPS);// 60hz

const SPRITES: [u8; 0x50] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const WAIT_EVENTS_KEY_PRESS: u16 = 0x0001;

pub struct CPU {
    registers: [u8; 16],
    pc: u16,
    reg_index: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    frame_buffer: [[u8; 64]; 32],
    memory: [u8; MEMORY_LEN],
    event_pump: EventPump,
    wait_events: u16,
    paused: bool,
    keyboard: Keyboard,
    display: Display,
    sound: Sound,
}

#[derive(Debug)]
struct Opcode(u16);

impl Opcode {
    fn x(&self) -> usize {
        ((self.0 & 0x0F00) >> 8) as usize
    }
    fn y(&self) -> usize {
        ((self.0 & 0x00F0) >> 4) as usize
    }
    fn n(&self) -> u8 {
        (self.0 & 0x000F) as u8
    }
    fn nn(&self) -> u8 {
        (self.0 & 0x00FF) as u8
    }
    fn nnn(&self) -> u16 {
        self.0 & 0x0FFF as u16
    }
}

impl CPU {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        CPU {
            registers: [0; 16],
            pc: 0,
            reg_index: 0,
            stack: vec![0u16; 64],
            delay_timer: 0u8,
            sound_timer: 0u8,
            frame_buffer: [[0; 64]; 32],
            memory: [0; 4 * 1024],
            event_pump: sdl_context.event_pump().unwrap(),
            wait_events: 0x0000,
            paused: false,
            keyboard: Keyboard::new(),
            display: Display::new(&sdl_context),
            sound: Sound::new(&sdl_context),
        }
    }

    pub fn init(&mut self, rom: &[u8]) {
        self.pc = MEMORY_PROGRAM_BASE;
        self.memory[0..SPRITES.len()].copy_from_slice(&SPRITES);
        self.memory[MEMORY_PROGRAM_BASE as usize..((MEMORY_PROGRAM_BASE as usize) + rom.len() - 1)].copy_from_slice(&rom[1..])
    }

    pub fn start(&mut self) -> ! {
        loop {
            let s = Instant::now();
            if let Some(ref event) = self.event_pump.poll_event() {
                match event {
                    Event::KeyDown { .. } | Event::KeyUp { .. } => {
                        self.keyboard.on_keyboard_event(event);
                    }
                    _ => {}
                }
            }
            if self.paused {
                self.on_paused();
            } else {
                self.exec_opcodes(16);
            }
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound.resume();
                self.delay_timer -= 1;
                if self.delay_timer <= 0 {
                    self.sound.pause();
                }
            }
            self.display.render();
            let (sleep, is_ovf) = MICROS_PER_FRAME.as_micros().overflowing_sub((Instant::now() - s).as_micros());
            if !is_ovf {
                thread::sleep(Duration::from_micros(sleep as u64));
            }
        }
    }

    fn exec_opcodes(&mut self, opcodes_per_frame: u8) {
        for _ in 0..opcodes_per_frame {
            let opcode = self.read_opcode();
            self.pc = self.pc + 0x0002;// point to next instruction
            self.exec_opcode(&opcode);
            if self.paused {
                self.stack.push(opcode.0);
                return;
            } else {}
        }
    }

    fn on_paused(&mut self) {
        let mut paused = true;
        if self.wait_events & WAIT_EVENTS_KEY_PRESS > 0 {
            if let Some(key) = self.keyboard.last_pressed_key() {
                if let Some(opcode) = self.stack.pop().map(|it| Opcode(it)) {
                    self.registers[opcode.x()] = key;
                }
                paused = false;
            };
        }
        self.paused = paused;
    }

    #[inline]
    fn read_opcode(&mut self) -> Opcode {
        let pc = self.pc;
        // 大端序
        let high = self.memory[pc as usize] as u16;
        let low = self.memory[(pc + 1) as usize] as u16;
        Opcode((high) << 8 | low)
    }

    pub fn exec_opcode(&mut self, opcode: &Opcode) {
        let x = opcode.x();
        let y = opcode.y();
        let n = opcode.n();
        let nn = opcode.nn();
        let nnn = opcode.nnn();
        let opcode = opcode.0;
        #[cfg(feature = "log_debug")]
        println!("{:04X}", opcode);
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => {
                        // 00E0: Clear the screen
                        self.display.clear();
                    }
                    0x00EE => {
                        // 00EE: Return from subroutine
                        self.pc = self.stack.pop().unwrap();
                    }
                    // 0NNN: Execute RCA 1802 machine language routine at address NNN
                    _ => self.unsupported_instruction(opcode)
                }
            }
            0x1000 => {
                // 1NNN: Jump to address NNN
                self.pc = nnn;
            }
            0x2000 => {
                // 2NNN: Call subroutine at address NNN
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            0x3000 => {
                // 3XNN: Skip the following instruction if the value of register VX equals NN
                if self.registers[x] == nn {
                    self.pc += 0x0002;
                };
            }
            0x4000 => {
                // 4XNN: Skip the following instruction if the value of register VX is not equal to NN
                if self.registers[x] != nn {
                    self.pc += 0x0002;
                }
            }
            0x5000 => {
                // 5XY0: Skip the following instruction if the value of register VX is equal to the value of register VY
                if self.registers[x] == self.registers[y] {
                    self.pc += 0x0002;
                }
            }
            0x6000 => {
                // 6XNN: Set VX to NN
                self.registers[x] = nn;
            }
            0x7000 => {
                // 7XNN: Add NN to VX
                let (val, _) = self.registers[x].overflowing_add(nn);
                self.registers[x] = val;
            }
            0x8000 => {
                match opcode & 0x000F {
                    0x0000 => {
                        // 8XY0: Set VX to the value in VY
                        self.registers[x] = self.registers[y];
                    }
                    0x0001 => {
                        // 8XY1: Set VX to VX OR VY
                        self.registers[x] |= self.registers[y];
                    }
                    0x0002 => {
                        // 8XY2: Set VX to VX AND VY
                        self.registers[x] &= self.registers[y];
                    }
                    0x0003 => {
                        // 8XY3: Set VX to VX XOR VY
                        self.registers[x] ^= self.registers[y];
                    }
                    0x0004 => {
                        // 8XY4: Add the value of register VY to register VX. Set VF to 01 if a carry occurs. Set VF to 00 if a carry does not occur
                        let (val, is_ovf) = self.registers[x].overflowing_add(self.registers[y]);
                        self.registers[x] = val;
                        self.registers[0x0F] = if is_ovf { 1 } else { 0 };
                    }
                    0x0005 => {
                        // 8XY5: Subtract the value of register VY from register VX. Set VF to 00 if a borrow occurs. Set VF to 01 if a borrow does not occur
                        let (val, is_ovf) = self.registers[x].overflowing_sub(self.registers[y]);
                        self.registers[x] = val;
                        self.registers[0x0F] = if is_ovf { 1 } else { 0 };
                    }
                    0x0006 => {
                        // 8XY6: Store the value of register VY shifted right one bit in register VX. Set register VF to the least significant bit prior to the shift
                        let val = self.registers[x];
                        self.registers[0x0F] = val & 0x01;
                        self.registers[x] = val >> 0x01;
                    }
                    0x0007 => {
                        // 8XY7: Set register VX to the value of VY minus VX. Set VF to 00 if a borrow occurs. Set VF to 01 if a borrow does not occur
                        let (val, is_ovf) = self.registers[y].overflowing_sub(self.registers[x]);
                        self.registers[x] = val;
                        self.registers[0x0F] = if is_ovf { 1 } else { 0 };
                    }
                    0x000E => {
                        // 8XYE: Store the value of register VY shifted left one bit in register VX. Set register VF to the most significant bit prior to the shift
                        let val = self.registers[x];
                        self.registers[0x0F] = val & 0x80;
                        self.registers[x] = val << 0x01;
                    }
                    _ => self.unsupported_instruction(opcode)
                }
            }
            0x9000 => {
                // 9XY0: Skip the following instruction if the value of register VX is not equal to the value of register VY
                if self.registers[x] != self.registers[y] {
                    self.pc += 0x0002;
                }
            }
            0xA000 => {
                // ANNN: Store memory address NNN in register I
                self.reg_index = nnn;
            }
            0xB000 => {
                // Jump to location nnn + V0. The program counter is set to nnn plus the value of V0.
                self.pc = (self.registers[0] as u16) + nnn;
            }
            0xC000 => {
                // CXNN	Set VX to a random number with a mask of NN
                self.registers[x] = rand::thread_rng().gen::<u8>() & nn;
            }
            0xD000 => {
                // DXYN: Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
                // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
                self.registers[0xF] = 0;
                for row in 0..n as u16 {
                    let mut sprite = self.memory[(self.reg_index + row) as usize];
                    for col in 0..8 {
                        // msb of the row is set the plot pixel
                        if sprite & 0b1000_0000u8 > 0 {
                            // draw given pixel at
                            if self.display.set_pixel(
                                self.registers[x] as usize + col as usize,
                                self.registers[y] as usize + row as usize,
                            ) {
                                self.registers[0xF] = 1
                            }
                        }
                        // left shift by one
                        sprite = sprite << 1;
                    }
                }
            }
            0xE000 => {
                match opcode & 0x00FF {
                    0x009E => {
                        // EX9E: Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
                        if self.keyboard.is_pressed(self.registers[x]) {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // EXA1: Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed
                        if !self.keyboard.is_pressed(self.registers[x]) {
                            self.pc += 2;
                        }
                    }
                    _ => self.unsupported_instruction(opcode)
                }
            }
            0xF000 => {
                match opcode & 0x00FF {
                    0x0007 => {
                        // FX07: Store the current value of the delay timer in register VX
                        self.registers[x] = self.delay_timer;
                    }
                    0x000A => {
                        // FX0A: Wait for a keypress and store the result in register VX
                        self.paused = true;
                        self.wait_events = self.wait_events | WAIT_EVENTS_KEY_PRESS;
                    }
                    0x0015 => {
                        // FX15: Set the delay timer to the value of register VX
                        self.delay_timer = self.registers[x];
                    }
                    0x0018 => {
                        // FX18: Set the sound timer to the value of register VX
                        self.sound_timer = self.registers[x];
                    }
                    0x001E => {
                        // FX1E: Add the value stored in register VX to register I
                        self.reg_index += (self.registers[x] as u16);
                    }
                    0x0029 => {
                        // FX29: Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX
                        self.reg_index = (self.registers[x] as u16) * 5;
                    }
                    0x0033 => {
                        // FX33: Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I + 1, and I + 2
                        let x = self.registers[x];
                        self.memory[self.reg_index as usize] = (x / 100);
                        self.memory[(self.reg_index + 1) as usize] = ((x / 10) % 10);
                        self.memory[(self.reg_index + 2) as usize] = (x % 10);
                    }
                    0x0055 => {
                        // FX55: Store the values of registers V0 to VX inclusive in memory starting at address I, I is set to I + X + 1 after operation²
                        for i in 0..=x {
                            self.memory[(self.reg_index) as usize] = self.registers[i];
                            self.reg_index += 1;
                        }
                    }
                    0x0065 => {
                        // FX65: Fill registers V0 to VX inclusive with the values stored in memory starting at address I, I is set to I + X + 1 after operation²
                        for i in 0..=x {
                            self.registers[i] = self.memory[(self.reg_index) as usize];
                            self.reg_index += 1;
                        }
                    }
                    _ => self.unsupported_instruction(opcode)
                }
            }
            _ => self.unsupported_instruction(opcode)
        }
    }

    fn unsupported_instruction(&self, opcode: u16) {
        panic!("Unsupported instruction: {:04X}", opcode);
    }
}


