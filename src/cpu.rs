mod memory;
mod registers;

use crate::display::DisplayChip8;
use memory::Memory;
use rand::{self, Rng};
use registers::Registers;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::time::Instant;

const SEC_TO_NANOS: u128 = 1_000_000_000;
const SCANCODES_KEYS: [Scancode; 16] = [
    Scancode::Num1,
    Scancode::Num2,
    Scancode::Num3,
    Scancode::Num4,
    Scancode::Q,
    Scancode::W,
    Scancode::E,
    Scancode::R,
    Scancode::A,
    Scancode::S,
    Scancode::D,
    Scancode::F,
    Scancode::Z,
    Scancode::X,
    Scancode::C,
    Scancode::V,
];

fn get_scancode_key(scancode: Scancode) -> Option<u8> {
    match scancode {
        Scancode::Num1 => Some(1),
        Scancode::Num2 => Some(2),
        Scancode::Num3 => Some(3),
        Scancode::Num4 => Some(0xC),
        Scancode::Q => Some(4),
        Scancode::W => Some(5),
        Scancode::E => Some(6),
        Scancode::R => Some(0xD),
        Scancode::A => Some(7),
        Scancode::S => Some(8),
        Scancode::D => Some(9),
        Scancode::F => Some(0xE),
        Scancode::Z => Some(0xA),
        Scancode::X => Some(0),
        Scancode::C => Some(0xB),
        Scancode::V => Some(0xF),
        _ => None,
    }
}

pub struct CPU {
    registers: Registers,
    memory: Memory,
    display: DisplayChip8,
    stack: Vec<u16>,
    keys: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
}

impl CPU {
    pub fn new(pixel_size: u32) -> Self {
        Self {
            registers: Registers::default(),
            memory: Memory::default(),
            display: DisplayChip8::new(pixel_size),
            stack: Vec::new(),
            keys: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn run(&mut self) {
        let sdl_context = self.display.canvas.window().subsystem().sdl();
        let mut events = sdl_context.event_pump().unwrap();
        let mut cpu_tick_acc = 0;
        let cpu_ticks_per_second = 700;
        let mut timer_ticks = 0;
        let timer_ticks_per_second = 60;
        let mut delta_time = 0;
        'gameloop: loop {
            let begin = Instant::now();
            cpu_tick_acc += delta_time;
            timer_ticks += delta_time;
            if cpu_tick_acc > (SEC_TO_NANOS / cpu_ticks_per_second) {
                self.tick();
                cpu_tick_acc = 0;
            }
            if timer_ticks > (SEC_TO_NANOS / timer_ticks_per_second) {
                self.tick_timers();
                timer_ticks = 0;
            }
            let keyboard_state = events.keyboard_state();
            for scancode in SCANCODES_KEYS {
                if let Some(key) = get_scancode_key(scancode) {
                    self.keys[key as usize] = keyboard_state.is_scancode_pressed(scancode);
                }
            }
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        break 'gameloop;
                    }
                    _ => (),
                }
            }

            let end = Instant::now();
            delta_time = end.duration_since(begin).as_nanos();
        }
    }

    fn tick(&mut self) {
        let instruction = self.fetch_instruction();
        self.decode_and_execute(instruction);
    }

    fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
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
        let first_nibble = (opcode & (0xF000)) >> 12;
        let second_nibble = (opcode & (0x0F00)) >> 8;
        let third_nibble = (opcode & (0x00F0)) >> 4;
        let fourth_nibble = (opcode & (0x000F)) >> 0;
        match (first_nibble, second_nibble, third_nibble, fourth_nibble) {
            (0x0, 0x0, 0xE, 0x0) => self.display.clear(),
            (0x0, 0x0, 0xE, 0xE) => self.pop_stack(),
            (0x1, _, _, _) => {
                let address = (second_nibble << 8) | (third_nibble << 4) | fourth_nibble;
                self.registers.set_program_counter(address);
            }
            (0x2, _, _, _) => {
                self.push_stack((second_nibble << 8) | (third_nibble << 4) | fourth_nibble)
            }
            (0x3, _, _, _) => self.skip_if_eq_register(
                second_nibble as u8,
                ((third_nibble << 4) | fourth_nibble) as u8,
            ),
            (0x4, _, _, _) => self.skip_if_not_eq_register(
                second_nibble as u8,
                ((third_nibble << 4) | fourth_nibble) as u8,
            ),
            (0x5, _, _, 0x0) => self.skip_if_reg_equals(second_nibble as u8, third_nibble as u8),
            (0x6, _, _, _) => self.registers.set_register(
                second_nibble as usize,
                ((third_nibble << 4) | fourth_nibble) as u8,
            ),
            (0x7, _, _, _) => self.add_to_register(
                second_nibble as u8,
                ((third_nibble << 4) | fourth_nibble) as u8,
            ),
            (0x8, _, _, 0x0) => self.set_register_value(second_nibble as u8, third_nibble as u8),
            (0x8, _, _, 0x1) => {
                self.or_registers(second_nibble as u8, third_nibble as u8, second_nibble as u8)
            }
            (0x8, _, _, 0x2) => {
                self.and_registers(second_nibble as u8, third_nibble as u8, second_nibble as u8)
            }
            (0x8, _, _, 0x3) => {
                self.xor_registers(second_nibble as u8, third_nibble as u8, second_nibble as u8)
            }
            (0x8, _, _, 0x4) => {
                self.add_registers(second_nibble as u8, third_nibble as u8, second_nibble as u8)
            }
            (0x8, _, _, 0x5) => self.substract_registers(
                second_nibble as u8,
                third_nibble as u8,
                second_nibble as u8,
            ),
            (0x8, _, _, 0x6) => self.right_shift(second_nibble as u8, third_nibble as u8),
            (0x8, _, _, 0x7) => self.substract_registers(
                third_nibble as u8,
                second_nibble as u8,
                second_nibble as u8,
            ),
            (0x8, _, _, 0xE) => self.left_shift(second_nibble as u8, third_nibble as u8),
            (0x9, _, _, 0x0) => self.skip_if_reg_not_eq(second_nibble as u8, third_nibble as u8),
            (0xA, _, _, _) => {
                self.registers
                    .set_index((second_nibble << 8) | (third_nibble << 4) | fourth_nibble);
            }
            (0xB, _, _, _) => {
                self.jump_address_offset((second_nibble << 8) | (third_nibble << 4) | fourth_nibble)
            }
            (0xC, _, _, _) => self.generate_random_number(
                second_nibble as u8,
                ((third_nibble << 4) | fourth_nibble) as u8,
            ),
            (0xD, _, _, _) => {
                self.draw_to_screen(second_nibble as u8, third_nibble as u8, fourth_nibble as u8)
            }
            (0xE, _, 0x9, 0xE) => self.skip_if_key_pressed(second_nibble as u8),
            (0xE, _, 0xA, 0x1) => self.skip_if_not_key_pressed(second_nibble as u8),
            (0xF, _, 0x0, 0x7) => self.set_reg_to_delay_timer(second_nibble as u8),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer(second_nibble as u8),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer(second_nibble as u8),
            (0xF, _, 0x1, 0xE) => self.add_to_index(second_nibble as u8),
            (0xF, _, 0x0, 0xA) => self.get_key(second_nibble as u8),
            (0xF, _, 0x2, 0x9) => self.set_index_to_font(second_nibble as u8),
            (0xF, _, 0x3, 0x3) => self.binary_coded_decimal_conversion(second_nibble as u8),
            (0xF, _, 0x5, 0x5) => self.store_in_memory(second_nibble as u8),
            (0xF, _, 0x6, 0x5) => self.load_from_memory(second_nibble as u8),
            _ => eprintln!("Incorrect opcode: {opcode:#x}"),
        }
    }

    fn pop_stack(&mut self) {
        let new_pc = self.stack.pop();
        if let Some(new_pc) = new_pc {
            self.registers.set_program_counter(new_pc);
        } else {
            eprintln!("Nothing in the stack to pop");
        }
    }

    fn push_stack(&mut self, address: u16) {
        self.stack.push(self.registers.get_program_counter());
        self.registers.set_program_counter(address);
    }

    fn skip_if_eq_register(&mut self, register: u8, value: u8) {
        let register_value = self.registers.get_register(register as usize);
        if let Some(register_value) = register_value {
            let skip = register_value == value;
            if skip {
                self.fetch_instruction();
            }
        } else {
            eprintln!("Invalid register {register}");
        }
    }

    fn skip_if_not_eq_register(&mut self, register: u8, value: u8) {
        let register_value = self.registers.get_register(register as usize);
        if let Some(register_value) = register_value {
            let skip = register_value != value;
            if skip {
                self.fetch_instruction();
            }
        } else {
            eprintln!("Invalid register {register}");
        }
    }

    fn skip_if_reg_equals(&mut self, reg_1: u8, reg_2: u8) {
        let first_value = self.registers.get_register(reg_1 as usize);
        let second_value = self.registers.get_register(reg_2 as usize);
        match (first_value, second_value) {
            (Some(first_value), Some(second_value)) => {
                let skip = first_value == second_value;
                if skip {
                    self.fetch_instruction();
                }
            }
            (None, None) => {
                eprintln!("Invalid registers: {reg_1}, {reg_2}")
            }
            (None, Some(_)) => eprintln!("Invalid register: {reg_1}"),
            (Some(_), None) => eprintln!("Invalid register: {reg_2}"),
        }
    }

    fn add_to_register(&mut self, register: u8, value: u8) {
        let register_value = self.registers.get_register(register as usize);
        if let Some(register_value) = register_value {
            self.registers
                .set_register(register as usize, register_value.wrapping_add(value));
        }
    }

    fn set_register_value(&mut self, dest_register: u8, source_register: u8) {
        let value = self
            .registers
            .get_register(source_register as usize)
            .unwrap();
        self.registers.set_register(dest_register as usize, value);
    }

    fn or_registers(&mut self, reg_1: u8, reg_2: u8, dest_register: u8) {
        let value_1 = self.registers.get_register(reg_1 as usize).unwrap();
        let value_2 = self.registers.get_register(reg_2 as usize).unwrap();
        let new_value = value_1 | value_2;
        self.registers
            .set_register(dest_register as usize, new_value);
    }

    fn and_registers(&mut self, reg_1: u8, reg_2: u8, dest_register: u8) {
        let value_1 = self.registers.get_register(reg_1 as usize).unwrap();
        let value_2 = self.registers.get_register(reg_2 as usize).unwrap();
        let new_value = value_1 & value_2;
        self.registers
            .set_register(dest_register as usize, new_value);
    }

    fn xor_registers(&mut self, reg_1: u8, reg_2: u8, dest_register: u8) {
        let value_1 = self.registers.get_register(reg_1 as usize).unwrap();
        let value_2 = self.registers.get_register(reg_2 as usize).unwrap();
        let new_value = value_1 ^ value_2;
        self.registers
            .set_register(dest_register as usize, new_value);
    }

    fn add_registers(&mut self, reg_1: u8, reg_2: u8, dest_register: u8) {
        let value_1 = self.registers.get_register(reg_1 as usize).unwrap();
        let value_2 = self.registers.get_register(reg_2 as usize).unwrap();
        let (new_value, did_overflow) = value_1.overflowing_add(value_2);
        self.registers
            .set_register(dest_register as usize, new_value);
        if did_overflow {
            self.registers.set_flag();
        } else {
            self.registers.reset_flag();
        }
    }

    fn substract_registers(&mut self, reg_1: u8, reg_2: u8, dest_register: u8) {
        let value_1 = self.registers.get_register(reg_1 as usize).unwrap();
        let value_2 = self.registers.get_register(reg_2 as usize).unwrap();
        let (new_value, did_borrow) = value_1.overflowing_sub(value_2);
        self.registers
            .set_register(dest_register as usize, new_value);
        if did_borrow {
            self.registers.reset_flag();
        } else {
            self.registers.set_flag();
        }
    }

    fn right_shift(&mut self, dest_register: u8, source_register: u8) {
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

    fn left_shift(&mut self, source_register: u8, dest_register: u8) {
        let source_value = self
            .registers
            .get_register(source_register as usize)
            .unwrap();
        let did_overflow = source_value >> 7 == 1;
        let new_value = source_value << 1;
        self.registers
            .set_register(dest_register as usize, new_value);
        if did_overflow {
            self.registers.set_flag();
        } else {
            self.registers.reset_flag();
        }
    }

    fn skip_if_reg_not_eq(&mut self, reg_1: u8, reg_2: u8) {
        let value_1 = self.registers.get_register(reg_1 as usize);
        let value_2 = self.registers.get_register(reg_2 as usize);
        match (value_1, value_2) {
            (Some(first_value), Some(second_value)) => {
                let skip = first_value != second_value;
                if skip {
                    self.fetch_instruction();
                }
            }
            (None, None) => {
                eprintln!("Invalid registers: {reg_1}, {reg_2}")
            }
            (None, Some(_)) => eprintln!("Invalid register: {reg_1}"),
            (Some(_), None) => eprintln!("Invalid register: {reg_2}"),
        }
    }

    fn jump_address_offset(&mut self, base_address: u16) {
        let offset = self.registers.get_register(0).unwrap() as u16;
        self.registers.set_program_counter(base_address + offset);
    }

    fn generate_random_number(&mut self, register: u8, mask: u8) {
        let random_number = rand::thread_rng().gen::<u8>() & 0xFF;
        self.registers
            .set_register(register as usize, random_number & mask);
    }

    fn draw_to_screen(&mut self, reg_x: u8, reg_y: u8, number_bytes: u8) {
        if let (Some(x_position), Some(y_position)) = (
            self.registers.get_register(reg_x as usize),
            self.registers.get_register(reg_y as usize),
        ) {
            let x_position = x_position & 63;
            let y_position = y_position & 31;
            let bytes = self
                .memory
                .get_slice(self.registers.get_index(), number_bytes as u16);
            let did_flip_on_pixel = self.display.draw(x_position, y_position, bytes);
            if did_flip_on_pixel {
                self.registers.set_flag();
            } else {
                self.registers.reset_flag();
            }
            self.display.refresh();
        } else {
            eprintln!("Invalid registers {reg_x} {reg_y}");
        }
    }

    fn skip_if_key_pressed(&mut self, register: u8) {
        let register_value = self.registers.get_register(register as usize).unwrap();
        let skip = self.keys[register_value as usize];
        if skip {
            self.fetch_instruction();
        }
    }

    fn skip_if_not_key_pressed(&mut self, register: u8) {
        let register_value = self.registers.get_register(register as usize).unwrap();
        let skip = !self.keys[register_value as usize];
        if skip {
            self.fetch_instruction();
        }
    }

    fn set_reg_to_delay_timer(&mut self, register: u8) {
        self.registers
            .set_register(register as usize, self.delay_timer);
    }

    fn set_delay_timer(&mut self, register: u8) {
        self.delay_timer = self.registers.get_register(register as usize).unwrap();
    }

    fn set_sound_timer(&mut self, register: u8) {
        self.sound_timer = self.registers.get_register(register as usize).unwrap();
    }

    fn add_to_index(&mut self, register: u8) {
        let register_value = self.registers.get_register(register as usize).unwrap() as u16;
        let index = self.registers.get_index();
        self.registers.set_index(index + register_value);
    }

    fn get_key(&mut self, register: u8) {
        let mut found_key = false;
        for (key, is_key_pressed) in self.keys.iter().enumerate() {
            if *is_key_pressed {
                self.registers.set_register(register as usize, key as u8);
                found_key = true;
                break;
            }
        }
        if !found_key {
            self.registers
                .set_program_counter(self.registers.get_program_counter().wrapping_sub(2));
        }
    }

    fn set_index_to_font(&mut self, register: u8) {
        let register_value = self.registers.get_register(register as usize).unwrap();
        let character = register_value & 0xF;
        let character_address = self.memory.get_font_address(character);
        self.registers.set_index(character_address);
    }

    fn binary_coded_decimal_conversion(&mut self, register: u8) {
        let register_value = self.registers.get_register(register as usize).unwrap();
        let first_digit = register_value / 100;
        let second_digit = (register_value / 10) % 10;
        let third_digit = register_value % 10;
        let base_address = self.registers.get_index();
        self.memory.set_value(base_address, first_digit);
        self.memory
            .set_value(base_address.wrapping_add(1), second_digit);
        self.memory
            .set_value(base_address.wrapping_add(2), third_digit);
    }

    fn store_in_memory(&mut self, register: u8) {
        let base_index = self.registers.get_index();
        for reg in 0..=register {
            self.memory.set_value(
                base_index + reg as u16,
                self.registers.get_register(reg as usize).unwrap(),
            );
        }
    }

    fn load_from_memory(&mut self, register: u8) {
        let base_index = self.registers.get_index();
        for reg in 0..=register {
            self.registers
                .set_register(reg as usize, self.memory.get_value(base_index + reg as u16));
        }
    }
}
