use std::{fs, process};
use std::io::Read;

use crate::chip8::cpu::CPU;

mod chip8;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("chip8 <Rom>");
        process::exit(-1)
    }
    let rom_file = &args[1];
    let mut rom_file = match fs::File::open(rom_file) {
        Ok(rom_file) => rom_file,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    };
    let mut rom = vec![0_u8];
    rom_file.read_to_end(&mut rom).expect("Load failed");
    let rom_slice = rom.as_slice();
    let mut cpu = CPU::new();
    cpu.init(rom_slice);
    cpu.start();
}
