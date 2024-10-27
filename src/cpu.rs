use crate::{instructions::arm::process_instruction_arm, not_implemented};
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
    User,
    FIQ,
    IRQ,
    Supervisor,
    Abort,
    Undefined,
    System,
}

// emulation of a ARMT7DMI CPU
pub struct CPU {
    cycles: u128,
    pub registers: [u32; 37],
    pub t: bool,  // true for THUMB mode, false for ARM mode
    pub mode: CPUMode,
    pub cnzv: u8,  // C, N, Z, and V flags, only the lower 4 bits are used
}

const COND_MASK: u32 = 0xF0000000;

impl CPU {
    pub fn new() -> CPU {
        return CPU {
            cycles: 0,
            registers: [0; 37],
            t: false,
            mode: CPUMode::User,
            cnzv: 0x00,
        }
    }

    pub fn cycle(&mut self) {
        if self.t
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
    pub fn check_condition(&self, instruction: u32) -> bool {
        let cond_flags: u8 = ((instruction & COND_MASK) >> 28).try_into().unwrap();  // should never panic
        if cond_flags == self.cnzv {
            return true;
        }
        else {
            return false;
        }
    }

    // utilities to alias the first 16 registers depending on the mode the CPU is currently in
    // note: since R15 is shared across all modes, I'll read and write to it directly in the code instead of using these methods
    pub fn register_read(&self, register: u32) -> u32 
    {
        let register: usize = register.try_into().unwrap();
        if self.mode == CPUMode::User || self.mode == CPUMode::System {
            return self.registers[register];
        }
        else if self.mode == CPUMode::FIQ {
            if register >= 8 {
                return self.registers[register + 8];
            }
            else {
                return self.registers[register];
            }
        }
        else if self.mode == CPUMode::IRQ {
            if register >= 13 {
                return self.registers[register + 10];
            }
            else {
                return self.registers[register];
            }
        }
        else if self.mode == CPUMode::Supervisor {
            if register >= 13 {
                return self.registers[register + 12];
            }
            else {
                return self.registers[register];
            }
        }
        else if self.mode == CPUMode::Abort {
            if register >= 13 {
                return self.registers[register + 14];
            }
            else {
                return self.registers[register];
            }
        }
        else {
            if register >= 13 {
                return self.registers[register + 16];
            }
            else {
                return self.registers[register];
            }
        }
    }

    pub fn register_write(&self, register: u32, value: u32) 
    {
        let register: usize = register.try_into().unwrap();
        if self.mode == CPUMode::User || self.mode == CPUMode::System {
            self.registers[register] = value;
        }
        else if self.mode == CPUMode::FIQ {
            if register >= 8 {
                self.registers[register + 8] = value;
            }
            else {
                self.registers[register] = value;
            }
        }
        else if self.mode == CPUMode::IRQ {
            if register >= 13 {
                self.registers[register + 10] = value;
            }
            else {
                self.registers[register] = value;
            }
        }
        else if self.mode == CPUMode::Supervisor {
            if register >= 13 {
                self.registers[register + 12] = value;
            }
            else {
                self.registers[register] = value;
            }
        }
        else if self.mode == CPUMode::Abort {
            if register >= 13 {
                self.registers[register] + 14 = value;
            }
            else {
                self.registers[register] = value;
            }
        }
        else {
            if register >= 13 {
                self.registers[register + 16] = value;
            }
            else {
                self.registers[register] = value;
            }
        }
    }
}