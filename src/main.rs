mod cpu;
mod display;

use cpu::CPU;
use std::env;
use std::fs;

fn main() {
    let args = env::args();
    let mut rom_name = String::new();
    for (argument_number, argument) in args.enumerate() {
        println!(" {} {} ", argument_number, argument);
        if argument_number == 1 {
            rom_name = argument;
        }
    }
    if !rom_name.is_empty() {
        let rom_data = fs::read(format!("roms/{}", rom_name)).unwrap();
        //for (address, byte) in rom_data.iter().enumerate() {
        //println!("{address:#x} {byte:#x}");
        //}
        let mut cpu = CPU::new(20);
        cpu.load_rom(&rom_data);
        cpu.run();
    }
}
