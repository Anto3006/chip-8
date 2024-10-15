pub struct Registers {
    general_registers: [u8; 16],
    index: u16,
    program_counter: u16,
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

    pub fn increase_program_counter(&mut self, amount: u16) {
        self.program_counter += amount;
    }
}
