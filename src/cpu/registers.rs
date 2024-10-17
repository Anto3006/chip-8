const INITIAL_PC: u16 = 0x200;

pub struct Registers {
    general_registers: [u8; 16],
    index: u16,
    program_counter: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            general_registers: [0; 16],
            index: 0,
            program_counter: INITIAL_PC,
        }
    }
}

impl Registers {
    pub fn set_flag(&mut self) {
        self.general_registers[0xF] = 0x1;
    }

    pub fn reset_flag(&mut self) {
        self.general_registers[0xF] = 0x0;
    }

    pub fn is_flag_set(&mut self) -> bool {
        return self.general_registers[0xF] == 1;
    }

    pub fn get_program_counter(&self) -> u16 {
        self.program_counter
    }

    pub fn set_program_counter(&mut self, value: u16) {
        self.program_counter = value;
    }

    pub fn increase_program_counter(&mut self, amount: u16) {
        self.program_counter += amount;
    }

    pub fn set_register(&mut self, register: usize, value: u8) {
        if register < 16 {
            self.general_registers[register] = value;
        }
    }
    pub fn get_register(&mut self, register: usize) -> Option<u8> {
        if register < 16 {
            Some(self.general_registers[register])
        } else {
            None
        }
    }

    pub fn get_index(&self) -> u16 {
        self.index
    }

    pub fn set_index(&mut self, value: u16) {
        self.index = value;
    }
}
