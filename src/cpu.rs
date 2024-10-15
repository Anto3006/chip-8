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
            0x5000..=0x5FFF => {
                if opcode & 0x000F == 0 {
                    let first_register = (opcode & 0x0F00) >> 8 as u8;
                    let second_register = (opcode & 0x00F0) >> 4 as u8;
                    println!(
                        "Skip next instruction if register {} is equal to register {}",
                        first_register, second_register
                    );
                } else {
                    println!("Invalid opcode {opcode:#x}");
                }
            }
            0x6000..=0x6FFF => {
                let register = (opcode & 0x0F00) >> 8 as u8;
                let value = (opcode & 0x00FF) as u8;
                println!("Store value  {} in register {}", value, register);
            }
            0x7000..=0x7FFF => {
                let register = (opcode & 0x0F00) >> 8 as u8;
                let value = (opcode & 0x00FF) as u8;
                println!("Adds value {} to register {}", value, register);
            }
            0x8000..=0x8FFF => {
                let dest_register = (opcode & 0x0F00) >> 8 as u8;
                let source_register = (opcode & 0x00F0) >> 4 as u8;
                match opcode & 0x000F {
                    0x0 => {
                        println!(
                            "Store value in register {} into register {}",
                            source_register, dest_register
                        );
                    }
                    0x1 => {
                        println!(
                            "Set value in register {} to {} OR {}",
                            dest_register, dest_register, source_register
                        );
                    }
                    0x2 => {
                        println!(
                            "Set value in register {} to {} AND {}",
                            dest_register, dest_register, source_register
                        );
                    }
                    0x3 => {
                        println!(
                            "Set value in register {} to {} XOR {}",
                            dest_register, dest_register, source_register
                        );
                    }
                    0x4 => {
                        println!(
                            "ADD the value in register {} to register {}. Sets carry flag if needed.",
                            source_register, dest_register
                        );
                    }
                    0x5 => {
                        println!(
                            "SUBSTRACT the value in register {} from register {}. Sets carry flag if needed.",
                            source_register, dest_register
                        );
                    }
                    0x6 => {
                        println!(
                            "Set value in register {} right shifted one place in {}. Set carry flag if needed.",
                            source_register, dest_register
                        );
                    }
                    0x7 => {
                        println!(
                            "Set the value in register {} to the value register {} minus the value in register {}. Sets carry flag if needed.",
                            dest_register, source_register, dest_register
                        );
                    }
                    0xE => {
                        println!(
                            "Set value in register {} left shifted one place in {}. Set carry flag if needed.",
                            source_register, dest_register
                        );
                    }
                    _ => println!("Invalid opcode {opcode:#x}"),
                }
            }
            0xA000..=0xAFFF => {
                let address = opcode & 0x0FFF;
                println!("Store address {} in index register", address);
            }
            0xD000..=0xDFFF => {
                let register_x_position = opcode & 0x0F00 >> 8 as u8;
                let register_y_position = opcode & 0x00F0 >> 4 as u8;
                let number_bytes = (opcode & 0x000F) as u8;
                println!("Draw a sprite in position pointed by registers {},{} with {} bytes of data starting at address pointed by the index register. Change flag if needed.", register_x_position, register_y_position, number_bytes);
            }

            _ => (),
        }
    }
}
