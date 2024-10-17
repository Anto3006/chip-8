use core::panic;
use std::usize;

const MEMORY_SIZE: usize = 4096;
const INITIAL_POSITION: usize = 0x200;

pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Default for Memory {
    fn default() -> Self {
        Self { memory: [0; 4096] }
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
}
