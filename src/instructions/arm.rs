use crate::{cpu::CPU, cpu::CPUMode, cpu::Registers::*, not_implemented};

// various mask to extract certain bits from the instruction
const B_3_0:   u32 = 0x0000000F;  // lowest 4 bits
const B_0:     u32 = 0x00000001;  // lowest bit
const B_24:    u32 = 0x01000000;  // bit 24
const B_23:    u32 = 0x00800000;  // bit 23
const B_23_0:  u32 = 0x00FFFFFF;  // lower 24 bits
const B_25:    u32 = 0x02000000;  // bit 25
const B_20:    u32 = 0x00100000;  // bit 20
const B_24_21: u32 = 0x03E00000;  // bits 24 to 21
const B_19_16: u32 = 0x000F0000;  // bits 19 to 16
const B_15_12: u32 = 0x0000F000;  // bits 15 to 12
const B_11_0:  u32 = 0x00000FFF;  // lowest 12 bits
const B_4:     u32 = 0x00000010;  // bit 4
const B_7:     u32 = 0x00000040;  // bit 7
const B_6_5:   u32 = 0x00000060;  // bits 6 and 5
const B_11_7:  u32 = 0x00000780;  // bits 11 to 7
const B_11_8:  u32 = 0x00000700;  // bits 11 to 8
const B_7_0:   u32 = 0x000000FF;  // bits 7 to 0

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

const ARM_DATA_OPCODES: [(u32, ProcFnArm); 16] = [
        (0x00000000, placeholder_arm),  // AND
        (0x00000001, placeholder_arm),  // EOR
        (0x00000002, placeholder_arm),  // SUB
        (0x00000003, placeholder_arm),  // RSB
        (0x00000004, placeholder_arm),  // ADD
        (0x00000005, placeholder_arm),  // ADC
        (0x00000006, placeholder_arm),  // SBC
        (0x00000007, placeholder_arm),  // RSC
        (0x00000008, placeholder_arm),  // TST
        (0x00000009, placeholder_arm),  // TEQ
        (0x0000000A, placeholder_arm),  // CMP
        (0x0000000B, placeholder_arm),  // CMN
        (0x0000000C, placeholder_arm),  // ORR
        (0x0000000D, placeholder_arm),  // MOV
        (0x0000000E, placeholder_arm),  // BIC
        (0x0000000F, placeholder_arm),  // MVN
];

type
const ARM_SHIFT_TYPES

pub fn process_instruction(cpu: &mut CPU, instruction: u32) {
    let mut handled = false;
    for (pattern, mask, handler) in ARM_OPCODES
    {
        if (instruction & mask) == pattern
        {
            handler(cpu, instruction);
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
    let i: u32 = instruction & B_25;
    let s: u32 = instruction & B_20;
    let rn: u32 = (instruction & B_19_16) >> 16;
    let rd: u32 = (instruction & B_15_12) >> 12;
    //let op2: u32 = instruction & B_11_0;
    let opcode: u32 = (instruction & B_24_21) >> 21;

    // resolve the second operand
    let mut op2;
    if i != 0 {
        // I flag is set, meaning the second operand is an immediate value
        // procedure: extend the 8 bit value to 32 bit, then rotate by twice the amount in the rotate bits
        let rotate: u32 = (instruction & B_11_8) >> 8;
        op2 = instruction & B_7_0;
        op2 = rotate_32bit(op2, rotate * 2);
    }
    else {
        // I flag is not set, meaning the second operand is a value in a register
        // procedure: get value from target register, determine shift amount and type, apply shift to value
        
        let op2_init_address = instruction & B_3_0;
        let op2_init_value = cpu.register_read(op2_init_address);

        // the shift amount can be an immediate value or loaded in from a register
        let bit4: u32 = instruction & B_4;
        let shift_amount: u32;
        if bit4 != 0 {
            // in this case, we load in the shift amount from the bottom 4 bytes of the register mentioned in bits 11 to 8
            let shift_register_address: u32 = (instruction & B_11_8) >> 8;
            let shift_register_value: u32 = cpu.register_read(shift_register_address.try_into().unwrap());
            shift_amount = shift_register_value & B_3_0;
        }
        else {
           // in this case the amount is determined from the instruction
           shift_amount = (instruction & B_11_7) >> 7; 
        }
        let shift_type: u32 = (instruction & B_6_5) >> 5;

    }
}

pub fn branch_and_exchange(cpu: &mut CPU, instruction: u32) {
    // handle registers in other CPU modes
    let rn: usize = (instruction & B_3_0) as usize;
    let t_bit: u32 = instruction & B_0;
    if rn == 15 {
        println!("[WARNING] Branch and exchange instruction into the program counter register (R15), undefined behavior!")
    }
    cpu.registers[R15] = cpu.register_read(rn);
    cpu.t = t_bit != 0;
}

pub fn branch(cpu: &mut CPU, instruction: u32) {
    let bit_24: u32 = instruction & B_24;
    if bit_24 != 0 {
        // link bit is set, copy old PC into R14
        if cpu.mode == CPUMode::User || cpu.mode == CPUMode::System {
            cpu.registers[R14] = cpu.registers[R15];
        }
        else if cpu.mode == CPUMode::FIQ {
            cpu.registers[R14_FIQ] = cpu.registers[R15];
        }
        else if cpu.mode == CPUMode::IRQ {
            cpu.registers[R14_IRQ] = cpu.registers[R15];
        }
        else if cpu.mode == CPUMode::Supervisor {
            cpu.registers[R14_SVC] = cpu.registers[R15];
        }
        else if cpu.mode == CPUMode::Abort {
            cpu.registers[R14_ABT] = cpu.registers[R15];
        }
        else if cpu.mode == CPUMode::Undefined {
            cpu.registers[R14_UND] = cpu.registers[R15];
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

pub fn rotate_32bit(value: u32, rotate: u32) -> u32
{
    let rotate = rotate % 32;
    return (value >> rotate) | (value << (32 - rotate));
}