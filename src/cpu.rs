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
// memory is included here, this mirrors the way it was manufactured in real life where the RAM is integrated into the CPU chip
pub struct CPU {
    cycles: u128,
    pub registers: [u32; 37],
    // memory
    pub bios: [u32; 4096],  // 16 KB in real life
    pub board_ram: [u32; 65536],  // 256 KB
    pub chip_ram: [u32; 8192],  // 32 KB
    pub palette_ram: [u32; 256],  // 1 KB
    pub video_ram: [u32; 24576],  // 96 KB
    pub obj_att: [u32; 256],  // 1 KB
    pub game_pak_ram: [u32; 16384],  // 64 KB
}

impl CPU {
    pub fn new() -> CPU {
        let mut init: [u32; 37] = [0; 37];
        init[Registers::CPSR] = 16; // 16 == binary for user mode
        CPU {
            cycles: 0,
            registers: init,
            bios: [0; 4096],  // TODO: load in bios into this
            board_ram: [0; 65536],
            chip_ram: [0; 8192],
            palette_ram: [0; 256],
            video_ram: [0; 24576],
            obj_att: [0; 256],
            game_pak_ram: [0; 16384],
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

    pub fn memory_read(&self, address: u32, r_type: u32) -> u32 {
        /* 
            reads memory from address in RAM
            if read_type is 0, a single byte is loaded and placed into the lower 8 bits
            if read_type is 1, a halfword is loaded and placed into the lower 16 bits
            if read_type is 2, a word is loaded and returend
            if word is false, a single byte is loaded and placed into the lower 8 bits of the return value, with the rest set to 0
        */

        // resolve the given byte address into word address and byte offset
        let (w_address, w_byte) = (address / 4, address % 4);
        // put out warning in console


        // resolve address for the different areas of memory and get value
        // going by the memory map on https://problemkaputt.de/gbatek.htm#gbamemorymap 
        let value: u32;
        if address >= 0x00000000 && address <= 0x00003FFF {
            // BIOS
            value = self.bios[w_address as usize];
        }
        else if address >= 0x02000000 && address <= 0x0203FFFF {
            // board RAM
            value = self.board_ram[(w_address - 0x00800000) as usize];
        }
        else if address >= 0x03000000 && address <= 0x03007FFF {
            // chip RAM
            value = self.chip_ram[(w_address - 0x00C00000) as usize];
        }
        else if address >= 0x04000000 && address <= 0x040003FE {
            // IO registers
            not_implemented!();
        }
        else if address >= 0x05000000 && address <= 0x050003FF {
            // BG/OBJ palette
            value = self.palette_ram[(w_address - 0x01400000) as usize];
        }
        else if address >= 0x06000000 && address <= 0x06017FFF {
            // VRAM
            value = self.video_ram[(w_address - 0x01800000) as usize];
        }
        else if address >= 0x07000000 && address <= 0x070003FF {
            // OBJ attributes
            value = self.obj_att[(w_address - 0x01C00000) as usize];
        }
        else if address >= 0x08000000 && address <= 0x09FFFFFF {
            // Game Pak wait state 0
            not_implemented!();
        }
        else if address >= 0x0A000000 && address <= 0x0BFFFFFF {
            // Game Pak wait state 1
            not_implemented!();
        }
        else if address >= 0x0C000000 && address <= 0x0DFFFFFF {
            // Game Pak wait state 2
            not_implemented!();
        }
        else if address >= 0x0E000000 && address <= 0x0E00FFFF {
            // Game Pak SRAM
            value = self.game_pak_ram[(w_address - 0x03800000) as usize];
        }
        else {
            panic!("Read attempt in unused area of memory! Address: {:x}", address);
        }

        match r_type {
            0 => {
                // shift the desired byte to the lowest position
                let shifted_value = value >> (8 * w_byte);
                // set the rest to zero and return
                return shifted_value & B_15_0;
            },
            1 => {
                // same as above, just with different mask
                let shifted_value = value >> (8 * w_byte);
                return shifted_value & B_15_0;
            }
            2 => {
                // we need to rotate the value such that the addressed byte ends up at position 0 to 7 in the return value
                // note: in contrast to half word loads, there is no masking or sign extend here
                // using the inbuilt Rust rotate here because there are no side effects on the processor flags
                return value.rotate_left(8 * w_byte);
            },
            _ => panic!("Invalid read type {} in memory read!", r_type)
        }
    }

    pub fn memory_write(&mut self, address: u32, w_type: u32, value: u32) {
        // writes to memory address in RAM
        // if write type is 0, a byte write is performed
        // value contains the byte in bits 0 to 7 and otherwise it's 0
        // if write type is 1, a halfword write is performed
        // value must contain the two bytes in bits 0 to 8, otherwise it's 0
        // if write type is 2, a word write is performed
        // the entire value parameter is written into memory

        // resolve the given byte address into word address and byte offset
        let (w_address, w_byte) = (address / 4, address % 4);
        // determine data to write and mask
        let write_data: u32;
        let write_mask: u32;
        match w_type {
            0 => {
                write_data = value << (8 * w_byte);  // shift data into correct position
                write_mask = !(0x000000FF << (8 * w_byte));  // create mask in correct position
            },
            1 => {
                if w_byte % 2 != 0 {
                    panic!("Write of halfword into non-halfword-aligned address {:x}!", address);
                }
                write_data = value << (8 * w_byte);
                write_mask = !(0x0000FFFF << (8 * w_byte));
            },
            2 => {
                if w_byte != 0 {
                    panic!("Write of word into non-word-aligned address {:x}!", address);
                }
                write_data = value;
                write_mask = 0x0;
            },
            _ => panic!("Invalid write mode {} in memory write!", w_type)
        }

        // write value into memory
        // get old data, null out the sections to overwrite, or with new value
        // going by the memory map on https://problemkaputt.de/gbatek.htm#gbamemorymap 
        if address >= 0x00000000 && address <= 0x00003FFF {
            // BIOS
            self.bios[w_address as usize] = (self.bios[w_address as usize] & write_mask) | write_data;
        }
        else if address >= 0x02000000 && address <= 0x0203FFFF {
            // board RAM
            self.board_ram[w_address as usize] = (self.board_ram[w_address as usize] & write_mask) | write_data;
        }
        else if address >= 0x03000000 && address <= 0x03007FFF {
            // chip RAM
            self.chip_ram[w_address as usize] = (self.chip_ram[w_address as usize] & write_mask) | write_data;
        }
        else if address >= 0x04000000 && address <= 0x040003FE {
            // IO registers
            not_implemented!();
        }
        else if address >= 0x05000000 && address <= 0x050003FF {
            // BG/OBJ palette
            self.palette_ram[w_address as usize] = (self.palette_ram[w_address as usize] & write_mask) | write_data;
        }
        else if address >= 0x06000000 && address <= 0x06017FFF {
            // VRAM
            self.video_ram[w_address as usize] = (self.video_ram[w_address as usize] & write_mask) | write_data;
        }
        else if address >= 0x07000000 && address <= 0x070003FF {
            // OBJ attributes
            self.obj_att[w_address as usize] = (self.obj_att[w_address as usize] & write_mask) | write_data;
        }
        else if address >= 0x08000000 && address <= 0x09FFFFFF {
            // Game Pak wait state 0
            not_implemented!();
        }
        else if address >= 0x0A000000 && address <= 0x0BFFFFFF {
            // Game Pak wait state 1
            not_implemented!();
        }
        else if address >= 0x0C000000 && address <= 0x0DFFFFFF {
            // Game Pak wait state 2
            not_implemented!();
        }
        else if address >= 0x0E000000 && address <= 0x0E00FFFF {
            // Game Pak SRAM
            self.game_pak_ram[w_address as usize] = (self.game_pak_ram[w_address as usize] & write_mask) | write_data;
        }
        else {
            panic!("Write attempt in unused area of memory! Address: {:x}", address);
        }
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