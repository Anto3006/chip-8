mod memory;
mod registers;

use crate::display::DisplayChip8;
use memory::Memory;
use rand::{self, Rng};
use registers::Registers;
use std::time::Duration;

pub struct CPU {
    registers: Registers,
    memory: Memory,
    display: DisplayChip8,
    stack: Vec<u16>,
}

impl CPU {
    pub fn new(pixel_size: u32) -> Self {
        Self {
            registers: Registers::default(),
            memory: Memory::default(),
            display: DisplayChip8::new(pixel_size),
            stack: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        let mut iter = 0;
        let time_seconds = 10;
        let loop_per_second = 70;
        loop {
            let instruction = self.fetch_instruction();
            self.decode_and_execute(instruction);
            iter += 1;
            if iter == time_seconds * loop_per_second {
                break;
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / loop_per_second));
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.memory.load(data);
    }

    fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.get_value(self.registers.get_program_counter());
        self.registers.increase_program_counter(1);
        byte
    }

    fn fetch_instruction(&mut self) -> u16 {
        let first_half = self.fetch_byte() as u16;
        let second_half = self.fetch_byte() as u16;
        let instruction = (first_half << 8) | second_half;
        instruction
    }
    fn decode_and_execute(&mut self, opcode: u16) {
        match opcode {
            0x00E0 => self.display.clear(),
            0x00EE => {
                let new_pc = self.stack.pop();
                if let Some(new_pc) = new_pc {
                    self.registers.set_program_counter(new_pc);
                } else {
                    eprintln!("Nothing in the stack to pop");
                }
            }
            0x1000..=0x1FFF => {
                let address = opcode & 0x0FFF;
                self.registers.set_program_counter(address);
            }
            0x2000..=0x2FFF => {
                self.stack.push(self.registers.get_program_counter());
                let address = opcode & 0x0FFF;
                self.registers.set_program_counter(address);
            }
            0x3000..=0x3FFF => {
                let register = (opcode & 0x0F00) >> 8 as u8;
                let register_value = self.registers.get_register(register as usize);
                if let Some(register_value) = register_value {
                    let value = (opcode & 0x0FF) as u8;
                    let skip = register_value == value;
                    if skip {
                        self.fetch_instruction();
                    }
                } else {
                    eprintln!("Invalid register {register}");
                }
            }
            0x4000..=0x4FFF => {
                let register = (opcode & 0x0F00) >> 8 as u8;
                let register_value = self.registers.get_register(register as usize);
                if let Some(register_value) = register_value {
                    let value = (opcode & 0x0FF) as u8;
                    let skip = register_value != value;
                    if skip {
                        self.fetch_instruction();
                    }
                } else {
                    eprintln!("Invalid register {register}");
                }
            }
            0x5000..=0x5FFF => {
                if opcode & 0x000F == 0 {
                    let first_register = (opcode & 0x0F00) >> 8 as u8;
                    let first_value = self.registers.get_register(first_register as usize);
                    let second_register = (opcode & 0x00F0) >> 4 as u8;
                    let second_value = self.registers.get_register(second_register as usize);
                    match (first_value, second_value) {
                        (Some(first_value), Some(second_value)) => {
                            let skip = first_value == second_value;
                            if skip {
                                self.fetch_instruction();
                            }
                        }
                        (None, None) => {
                            eprintln!("Invalid registers: {first_register}, {second_register}")
                        }
                        (None, Some(_)) => eprintln!("Invalid register: {first_register}"),
                        (Some(_), None) => eprintln!("Invalid register: {second_register}"),
                    }
                } else {
                    println!("Invalid opcode {opcode:#x}");
                }
            }
            0x6000..=0x6FFF => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;
                self.registers.set_register(register as usize, value);
            }
            0x7000..=0x7FFF => {
                let register = (opcode & 0x0F00) >> 8;
                let value = (opcode & 0x00FF) as u8;
                let register_value = self.registers.get_register(register as usize);
                if let Some(register_value) = register_value {
                    self.registers
                        .set_register(register as usize, register_value.wrapping_add(value));
                }
            }
            0x8000..=0x8FFF => {
                let dest_register = (opcode & 0x0F00) >> 8;
                let source_register = (opcode & 0x00F0) >> 4;
                match opcode & 0x000F {
                    0x0 => {
                        let value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        self.registers.set_register(dest_register as usize, value);
                    }
                    0x1 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let dest_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let new_value = dest_value | source_value;
                        self.registers
                            .set_register(dest_register as usize, new_value);
                    }
                    0x2 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let dest_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let new_value = dest_value & source_value;
                        self.registers
                            .set_register(dest_register as usize, new_value);
                    }
                    0x3 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let dest_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let new_value = dest_value ^ source_value;
                        self.registers
                            .set_register(dest_register as usize, new_value);
                    }
                    0x4 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let dest_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let (new_value, did_overflow) = dest_value.overflowing_add(source_value);
                        self.registers
                            .set_register(dest_register as usize, new_value);
                        if did_overflow {
                            self.registers.set_flag();
                        } else {
                            self.registers.reset_flag();
                        }
                    }
                    0x5 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let dest_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let (new_value, did_borrow) = dest_value.overflowing_sub(source_value);
                        self.registers
                            .set_register(dest_register as usize, new_value);
                        if did_borrow {
                            self.registers.reset_flag();
                        } else {
                            self.registers.set_flag();
                        }
                    }
                    0x6 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let did_overflow = source_value & 1 == 1;
                        let new_value = source_value >> 1;
                        self.registers
                            .set_register(dest_register as usize, new_value);
                        if did_overflow {
                            self.registers.set_flag();
                        } else {
                            self.registers.reset_flag();
                        }
                    }
                    0x7 => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let dest_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let (new_value, did_borrow) = source_value.overflowing_sub(dest_value);
                        self.registers
                            .set_register(dest_register as usize, new_value);
                        if did_borrow {
                            self.registers.reset_flag();
                        } else {
                            self.registers.set_flag();
                        }
                    }
                    0xE => {
                        let source_value = self
                            .registers
                            .get_register(source_register as usize)
                            .unwrap();
                        let did_overflow = source_value & 0x80 == 8;
                        let new_value = source_value << 1;
                        self.registers
                            .set_register(dest_register as usize, new_value);
                        if did_overflow {
                            self.registers.set_flag();
                        } else {
                            self.registers.reset_flag();
                        }
                    }
                    _ => println!("Invalid opcode {opcode:#x}"),
                }
            }
            0x9000..=0x9FFF => {
                if opcode & 0x000F == 0 {
                    let first_register = (opcode & 0x0F00) >> 8 as u8;
                    let first_value = self.registers.get_register(first_register as usize);
                    let second_register = (opcode & 0x00F0) >> 4 as u8;
                    let second_value = self.registers.get_register(second_register as usize);
                    match (first_value, second_value) {
                        (Some(first_value), Some(second_value)) => {
                            let skip = first_value != second_value;
                            if skip {
                                self.fetch_instruction();
                            }
                        }
                        (None, None) => {
                            eprintln!("Invalid registers: {first_register}, {second_register}")
                        }
                        (None, Some(_)) => eprintln!("Invalid register: {first_register}"),
                        (Some(_), None) => eprintln!("Invalid register: {second_register}"),
                    }
                } else {
                    println!("Invalid opcode {opcode:#x}");
                }
            }
            0xA000..=0xAFFF => {
                let address = opcode & 0x0FFF;
                self.registers.set_index(address);
            }
            0xB000..=0xBFFF => {
                let offset = self.registers.get_register(0).unwrap() as u16;
                let base_address = opcode & 0x0FFF;
                self.registers.set_program_counter(base_address + offset);
            }
            0xC000..=0xCFFF => {
                let register = opcode & 0x0F00;
                let mask = (opcode & 0x00FF) as u8;
                let random_number = rand::thread_rng().gen::<u8>() & 0xFF;
                self.registers
                    .set_register(register as usize, random_number & mask);
            }
            0xD000..=0xDFFF => {
                let register_x_position = (opcode & 0x0F00) >> 8;
                let register_y_position = (opcode & 0x00F0) >> 4;
                let number_bytes = (opcode & 0x000F) as u16;
                if let (Some(x_position), Some(y_position)) = (
                    self.registers.get_register(register_x_position as usize),
                    self.registers.get_register(register_y_position as usize),
                ) {
                    let x_position = x_position & 63;
                    let y_position = y_position & 31;
                    let bytes = self
                        .memory
                        .get_slice(self.registers.get_index(), number_bytes);
                    let did_flip_on_pixel = self.display.draw(x_position, y_position, bytes);
                    if did_flip_on_pixel {
                        self.registers.set_flag();
                    } else {
                        self.registers.reset_flag();
                    }
                } else {
                    eprintln!("Invalid registers {register_x_position} {register_y_position}");
                }
            }

            _ => (),
        }
    }
}
