use core::panic;
use std::usize;

pub struct Memory {
    memory: [u8; 4096],
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
}
