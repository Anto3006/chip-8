mod memory;
mod registers;

use memory::Memory;
use registers::Registers;

struct CPU {
    registers: Registers,
    memory: Memory,
}

impl CPU {
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.get_value(self.registers.get_program_counter());
        self.registers.increase_program_counter(1);
        byte
    }

    fn fetch_instruction(&mut self) -> u16 {
        let first_half = self.fetch_byte() as u16;
        let second_half = self.fetch_byte() as u16;
        (first_half << 8) | second_half
    }

    fn decode_instruction(opcode: u16) {
        match opcode {
            0x00E0 => println!("Clear the screen"),
            0x00EE => println!("Return from subroutine"),
            0x1000..=0x1FFF => {
                let address = opcode & 0x0FFF;
                println!("Jump to {}", address);
            }
            0x2000..=0x2FFF => {
                let address = opcode & 0x0FFF;
                println!("Execute subroutine in {}", address);
            }
            0x3000..=0x3FFF => {
                let register = (opcode & 0x0F00) >> 8 as u8;
                let value = (opcode & 0x0FF) as u8;
                println!(
                    "Skip next instruction if register {} is equal to {}",
                    register, value
                );
            }
            0x4000..=0x4FFF => {
                let register = (opcode & 0x0F00) >> 8 as u8;
                let value = (opcode & 0x0FF) as u8;
                println!(
                    "Skip next instruction if register {} is not equal to {}",
                    register, value
                );
            }
            _ => (),
        }
    }
}
