use crate::{instructions::arm::process_instruction, not_implemented};
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

    pub fn cycle(&self) {
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
                process_instruction(self, instruction);
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
}