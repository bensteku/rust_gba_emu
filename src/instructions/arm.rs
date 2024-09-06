use crate::{cpu::CPU, cpu::CPUMode, cpu::Registers::*, not_implemented};

// various mask to extract certain bits from the instruction
const B_3_0: u32 = 0x0000000F;  // lowest 4 bits
const B_0: u32 = 0x00000001; // lowest bit
const B_24: u32 = 0x0100000; // bit 24
const B_23: u32 = 0x0080000; // bit 23
const B_23_0: u32 = 0x00FFFFFF; // lower 24 bits

// lookup table for opcodes and their handling functions
// pattern, mask, handler function
type ProcFnArm = fn(&mut CPU, u32);
pub fn placeholder_arm(cpu: &mut CPU, opcode: u32) {
    not_implemented!();
}
const ARM_OPCODES: [(u32, u32, ProcFnArm); 15] = [
        (0x00000000, 0x0C000000, data_processing),  // data processing
        (0x00000090, 0x0FC000F0, placeholder_arm),  // multiply
        (0x00800090, 0x0F8000F0, placeholder_arm),  // multiply long
        (0x01000090, 0x0FB00FF0, placeholder_arm),  // single data swap
        (0x013FFF10, 0x0FFFFFF0, branch_and_exchange),  // branch and exchange
        (0x00000090, 0x0E400F90, placeholder_arm),  // halfword data transfer: register offset
        (0x00400090, 0x0E400090, placeholder_arm),  // halfword data transfer: immediate offset
        (0x04000000, 0x0C000000, placeholder_arm),  // single data transfer
        (0x06000010, 0x0E000010, placeholder_arm),  // undefined
        (0x08000000, 0x0E000000, placeholder_arm),  // block data transfer
        (0x0A000000, 0x0E000000, placeholder_arm),  // branch
        (0x0C000000, 0x0E000000, placeholder_arm),  // coprocessor data transfer
        (0x0E000000, 0x0F000000, placeholder_arm),  // coprocessor data operation
        (0x0E000010, 0x0F000010, placeholder_arm),  // coprocessor register transfer
        (0x0F000000, 0x0F000000, placeholder_arm),  // software interrupt
    ];

pub fn process_instruction(cpu: &mut CPU, instruction: u32) {
    let mut handled = false;
    for (pattern, mask, handler) in ARM_OPCODES
    {
        if (instruction & mask) == pattern
        {
            handler(instruction);
            handled = true;
            break;
        }
    }
    if !handled
    {
        println!("Unknown instruction detected!");
        println!("Instruction occured at {}.", 0);
        println!("Instruction binary: {:b}", instruction);
    }
}

pub fn data_processing(cpu: &mut CPU, instruction: u32) {
    return not_implemented!();   
}

pub fn branch_and_exchange(cpu: &mut CPU, instruction: u32) {
    let rn: usize = (instruction & B_3_0) as usize;
    let t_bit: u32 = instruction & B_0;
    if rn == 15 {
        println!("Branch and exchange instruction into the program counter register (R15), undefined behavior!")
    }
    cpu.registers[15] = cpu.registers[rn];
    cpu.t = t_bit != 0;
}

pub fn branch(cpu: &mut CPU, instruction: u32) {
    let bit_24: u32 = instruction & B_24;
    if bit_24 != 0 {
        // link bit is set, copy old PC into R14
        if cpu.mode == CPUMode::System || cpu.mode == CPUMode::User {
            cpu.registers[R14] = cpu.registers[R15];
        }
        else if cpu.mode == CPUMode::FIQ {
            cpu.registers[R14_FIQ] = cpu.registers[15];
        }
        else if cpu.mode == CPUMode::IRQ {
            cpu.registers[R14_IRQ] = cpu.registers[15];
        }
        else if cpu.mode == CPUMode::Supervisor {
            cpu.registers[R14_SVC] = cpu.registers[15];
        }
        else if cpu.mode == CPUMode::Abort {
            cpu.registers[R14_ABT] = cpu.registers[15];
        }
        else if cpu.mode == CPUMode::Undefined {
            cpu.registers[R14_UND] = cpu.registers[15];
        }
    }
    let mut offset: u32 = instruction & B_23_0;
    offset = offset << 2;
    let sign = offset & B_23;
    if sign != 0 {
        // negative number
        offset = offset | 0xFF000000;
    }
    else {
        offset = offset | 0x00000000;
    }
    cpu.registers[R15] += offset;
}