use crate::{instructions::arm::process_instruction_arm, not_implemented, instructions::masks_32bit::*, util::*};
use std::ops::Index;
use std::ops::IndexMut;

// reference for the registers
// this allows for easier array access into the CPU's register array
pub enum Registers {
    // Regular registers
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    // r8 to r12 are not available in THUMB mode
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    // Stack pointer
    R13 = 13,
    // Subroutine link register
    R14 = 14,
    // Program counter register
    R15 = 15,  // in ARM mode: bits 31:2 -> PC, bits 1:0 -> 0, in THUMB mode: 31:1 -> PC, bit 0 -> 0
    // Banked registers
    // Fast interrupt request mode
    R8_FIQ = 16,
    R9_FIQ = 17,
    R10_FIQ = 18,
    R11_FIQ = 19,
    R12_FIQ = 20,
    R13_FIQ = 21,
    R14_FIQ = 22,
    // Interrupt request mode
    R13_IRQ = 23,
    R14_IRQ = 24,
    // Supervisor
    R13_SVC = 25,
    R14_SVC = 26,
    // Abort
    R13_ABT = 27,
    R14_ABT = 28,
    // Undefined
    R13_UND = 29,
    R14_UND = 30,
    // Status registers
    // the CPSR follows the following format
    // bits 31 to 28: NZCV flags
    // bit 27 to 8: reserved
    // bits 7 to 0: IFT flags, then mode signature
    CPSR = 31,
    SPSR_FIQ = 32,
    SPSR_SVC = 33,
    SPSR_ABT = 34,
    SPSR_IRQ = 35,
    SPSR_UND = 36,
}
// define handy aliases
pub const SP: Registers = Registers::R13;
pub const LR: Registers = Registers::R14;
pub const SP_FIQ: Registers = Registers::R13_FIQ;
pub const LR_FIQ: Registers = Registers::R14_FIQ;
pub const SP_IRQ: Registers = Registers::R13_IRQ;
pub const LR_IRQ: Registers = Registers::R14_IRQ;
pub const SP_SVC: Registers = Registers::R13_SVC;
pub const LR_SVC: Registers = Registers::R14_SVC;
pub const SP_ABT: Registers = Registers::R13_ABT;
pub const LR_ABT: Registers = Registers::R14_ABT;
pub const SP_UND: Registers = Registers::R13_UND;
pub const LR_UND: Registers = Registers::R14_UND;


// these impls are necessary for the enum above to work with array accesses
impl<T> Index<Registers> for [T] {
    type Output = T;

    fn index(&self, index: Registers) -> &Self::Output {
        let idx = index as usize;
        &self[idx]
    }
}

impl<T> IndexMut<Registers> for [T] {
    fn index_mut(&mut self, index: Registers) -> &mut T {
        let idx = index as usize;
        &mut self[idx]
    }
}

#[derive(PartialEq)]
pub enum CPUMode {
    // numbers from CPU manual p.35
    User = 0b10000,
    FIQ = 0b10001,
    IRQ = 0b10010,
    Supervisor = 0b10011,
    Abort = 0b10111,
    Undefined = 0b11011,
    System = 0b11111,
}

#[repr(u32)]
pub enum ConditionFlags {
    V = 0x1000000,
    C = 0x2000000,
    Z = 0x4000000,
    N = 0x8000000,
}

// emulation of a ARMT7DMI CPU
pub struct CPU {
    cycles: u128,
    pub registers: [u32; 37],
}

impl CPU {
    pub fn new() -> CPU {
        let mut init: [u32; 37] = [0; 37];
        init[Registers::CPSR] = 16; // 16 == binary for user mode
        CPU {
            cycles: 0,
            registers: init
        }
    }

    pub fn cycle(&mut self) {
        if self.get_state()
        {
            not_implemented!();
        }
        else
        {
            let instruction: u32 = 0x00000000;  // TODO: replace with read
            // check if condition flags in instruction match with CPU state
            // if not then ignore the instruction
            if self.check_condition(instruction) {
                process_instruction_arm(self, instruction);
            }  
        }
    }

    // checks the condition of an instruction with the state of the CPU, this is only for ARM mode
    #[inline]
    pub fn check_condition(&self, instruction: u32) -> bool {
        if instruction & B_31_28 == self.get_all_condition_flags() {
            return true;
        }
        else {
            return false;
        }
    }

    // utilities to extract and set information in the CPSR
    pub fn get_state(&self) -> bool {
        // true: THUMB state, false: ARM state
        return self.registers[Registers::CPSR] & B_5 != 0;
    }

    pub fn set_state(&mut self, set: bool) {
        // same as above
        if set {
            self.registers[Registers::CPSR] = self.registers[Registers::CPSR] | (1 << 5);
        }
        else {
            self.registers[Registers::CPSR] = self.registers[Registers::CPSR] & !(1 << 5);
        }
    }

    pub fn get_fiq_disable(&self) -> bool {
        return self.registers[Registers::CPSR] & B_6 != 0;
    }

    pub fn set_fiq_disable(&mut self, set: bool) {
        if set {
            self.registers[Registers::CPSR] = self.registers[Registers::CPSR] | (1 << 6);
        }
        else {
            self.registers[Registers::CPSR] = self.registers[Registers::CPSR] & !(1 << 6);
        }
    }

    pub fn get_irq_disable(&self) -> bool {
        return self.registers[Registers::CPSR] & B_7 != 0;
    }

    pub fn set_irq_disable(&mut self, set: bool) {
        if set {
            self.registers[Registers::CPSR] = self.registers[Registers::CPSR] | (1 << 7);
        }
        else {
            self.registers[Registers::CPSR] = self.registers[Registers::CPSR] & !(1 << 7);
        }
    }

    pub fn get_mode(&self) -> CPUMode {
        let mode: u32 = self.registers[Registers::CPSR] & B_4_0;
        match mode {
            0b10000 => CPUMode::User,
            0b10001 => CPUMode::FIQ,
            0b10010 => CPUMode::IRQ,
            0b10011 => CPUMode::Supervisor,
            0b10111 => CPUMode::Abort,
            0b11011 => CPUMode::Undefined,
            0b11111 => CPUMode::System,
            _       => panic!("Invalid mode bits in CPSR!")
        }
    }

    pub fn set_mode(&mut self, mode: CPUMode) {
        match mode {
            CPUMode::User       => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b10000),
            CPUMode::FIQ        => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b10001),
            CPUMode::IRQ        => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b10010),
            CPUMode::Supervisor => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b10011),
            CPUMode::Abort      => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b10111),
            CPUMode::Undefined  => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b11011),
            CPUMode::System     => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 0, 4, 0b11111),
        }
    }

    pub fn get_condition_flag(&self, flag: ConditionFlags) -> bool {
        return self.registers[Registers::CPSR] & (flag as u32) != 0;
    }

    pub fn set_condition_flag(&mut self, flag: ConditionFlags, set: bool) {
        match flag {
            ConditionFlags::V  => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 28, 28, set as u32),
            ConditionFlags::C  => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 29, 29, set as u32),
            ConditionFlags::Z  => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 30, 30, set as u32),
            ConditionFlags::N  => self.registers[Registers::CPSR] = set_bits_in_range(self.registers[Registers::CPSR], 31, 31, set as u32),
        }
    }

    pub fn get_all_condition_flags(&self) -> u32 {
        return self.registers[Registers::CPSR] & 0xF0000000;
    }

    // utilities to alias the first 16 registers depending on the mode the CPU is currently in
    // further, using 16 will get you the CPSR, 17 the SPSR of the mode you're currently in
    // extra note: do not use the enum aliases from above with this!!!!
    pub fn register_read(&self, register: u32) -> u32 
    {
        let register: usize = register.try_into().unwrap();
        if register == 15 {
            return self.registers[15];  // special case for R15, as it's shared across all modes
        }
        else if register == 16 {
            return self.registers[Registers::CPSR];  // special case for number 16, as it's supposed to be the CPSR
        }
        if register > 17 {
            panic!("Invalid register number!")
        }
        match self.get_mode() {
            // FIQ, IRQ, Supervisor, Undefined and Abort have a special case for their SPSR
            CPUMode::User | CPUMode::System => return self.registers[register],
            CPUMode::FIQ => return if register == 17 {return self.registers[Registers::SPSR_FIQ]} else if register >= 8 {self.registers[register + 8]} else {self.registers[register]},
            CPUMode::IRQ => return if register == 17 {return self.registers[Registers::SPSR_IRQ]} else if register >= 13 {self.registers[register + 10]} else {self.registers[register]},
            CPUMode::Supervisor => return if register == 17 {return self.registers[Registers::SPSR_SVC]} else if register >= 13 {self.registers[register + 12]} else {self.registers[register]},
            CPUMode::Abort => return if register == 17 {return self.registers[Registers::SPSR_ABT]} else if register >= 13 {self.registers[register + 14]} else {self.registers[register]},
            CPUMode::Undefined => return if register == 17 {return self.registers[Registers::SPSR_UND]} else if register >= 13 {self.registers[register + 16]} else {self.registers[register]},
        }
    }

    pub fn register_write(&mut self, register: u32, value: u32) 
    {
        let register: usize = register.try_into().unwrap();
        if register == 15 {
            self.registers[15] = value;  // special case for R15, as it's shared across all modes
        }
        else if register == 16 {
            self.registers[16] = value;  // special case for element 16, as it's the CPSR
        }
        match self.get_mode() {
            CPUMode::User | CPUMode::System => self.registers[register] = value,
            CPUMode::FIQ => if register == 17 {self.registers[Registers::SPSR_FIQ] = value} else if register >= 8 {self.registers[register + 8] = value} else {self.registers[register] = value},
            CPUMode::IRQ => if register == 17 {self.registers[Registers::SPSR_IRQ] = value} else if register >= 13 {self.registers[register + 10] = value} else {self.registers[register] = value},
            CPUMode::Supervisor => if register == 17 {self.registers[Registers::SPSR_SVC] = value} else if register >= 13 {self.registers[register + 12] = value} else {self.registers[register] = value},
            CPUMode::Abort => if register == 17 {self.registers[Registers::SPSR_ABT] = value} else if register >= 13 {self.registers[register + 14] = value} else {self.registers[register] = value},
            CPUMode::Undefined => if register == 17 {self.registers[Registers::SPSR_UND] = value} else if register >= 13 {self.registers[register + 16] = value} else {self.registers[register] = value},
        }
    }
}