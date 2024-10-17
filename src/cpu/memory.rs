use core::panic;
use std::usize;

const MEMORY_SIZE: usize = 4096;
const INITIAL_POSITION: usize = 0x200;
const FONT: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        let mut memory = [0; 4096];
        memory[0x050..=0x09F].copy_from_slice(&FONT);
        Self { memory }
    }
}

impl Memory {
    pub fn get_value(&self, address: u16) -> u8 {
        if address < 4096 {
            self.memory[address as usize]
        } else {
            panic!("Invalid address");
        }
    }

    pub fn set_value(&mut self, address: u16, value: u8) {
        if address < 4096 {
            self.memory[address as usize] = value;
        } else {
            panic!("Invalid address");
        }
    }

    pub fn get_slice(&self, address: u16, number_bytes: u16) -> &[u8] {
        &self.memory[address as usize..(address + number_bytes) as usize]
    }

    pub fn load(&mut self, data: &[u8]) {
        for (position, value) in data.iter().enumerate() {
            if position < MEMORY_SIZE - INITIAL_POSITION {
                self.memory[position + INITIAL_POSITION] = *value;
            }
        }
    }

    pub fn get_font_address(&self, value: u8) -> u16 {
        if value <= 0xF {
            (0x50 + 5 * value) as u16
        } else {
            panic!("Invalid character");
        }
    }
}
