use crate::{cpu::{CPUMode, ConditionFlags, Registers::*, CPU}, 
            instructions::masks_32bit::*, 
            not_implemented,
            instructions::basic_ops::*};

type ProcFnThumb = fn(&mut CPU, u32);
pub fn placeholder_thumb(cpu: &mut CPU, opcode: u32) {
    not_implemented!();
}
const THUMB_OPCODES: [(u16, u16, ProcFnThumb); 19] = [
        (0x0000, 0xE000, move_shifted_register),  // move shifted register
        (0x1800, 0xF800, add_subtract),  // add/subtract
        (0x2000, 0xE000, placeholder_thumb),  // move/compare/add/subtract immediate
        (0x4000, 0xFC00, placeholder_thumb),  // alu operations
        (0x4400, 0xFC00, placeholder_thumb),  // hi register operations/branch exchange
        (0x4800, 0xF800, placeholder_thumb),  // pc relative load
        (0x5000, 0xF200, placeholder_thumb),  // load/store with register offset
        (0x5200, 0xF200, placeholder_thumb),  // load/store sign-extended byte/halfword
        (0x6000, 0xE000, placeholder_thumb),  // load/store with immediate offset
        (0x8000, 0xF000, placeholder_thumb),  // load/store halfword
        (0x9000, 0xF000, placeholder_thumb),  // sp-relative load/store
        (0xA000, 0xF000, placeholder_thumb),  // load address
        (0xB000, 0xFF00, placeholder_thumb),  // add offset to stack pointer
        (0xB400, 0xF600, placeholder_thumb),  // push/pop registers
        (0xC000, 0xF000, placeholder_thumb),  // multiple load/store
        (0xD000, 0xF000, placeholder_thumb),  // conditional branch
        (0xDF00, 0xFF00, placeholder_thumb),  // software interrupt
        (0xE000, 0xF800, placeholder_thumb),  // uncoditional branch
        (0xF000, 0xF000, placeholder_thumb),  // long branch with link
    ];

pub fn process_instruction_thumb(cpu: &mut CPU, instruction: u16) {
    let mut handled = false;
    for (pattern, mask, handler) in THUMB_OPCODES
    {
        if (instruction & mask) == pattern
        {
            handler(cpu, instruction as u32);
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

pub fn move_shifted_register(cpu: &mut CPU, instruction: u32) {
    // p.111

    // extract OP code, offset and registers
    let opcode = (instruction & B_12_11) >> 11;
    let offset5 = (instruction & B_10_6) >> 6;
    let rs = (instruction & B_5_3) >> 3;
    let rd = instruction & B_2_0;

    // execute operations according to opcode
    let source_value = cpu.register_read(rs);
    if opcode == 0 {
        // left shift
        let carry;
        if offset5 != 0 {
            carry = false;
        }
        else {
            carry = ((source_value >> (32 - offset5)) & B_0) != 0;
        }
        let result = source_value << offset5;
        cpu.register_write(rd, result);
        arithmetic_flag_helper(cpu, true, carry, false, result);
    }
    else if opcode == 1 {
        // right shift
        let carry;
        let result;
        if offset5 != 0 {
            result = source_value >> offset5;
            carry = ((source_value >> (offset5 - 1)) & B_0) != 0;
        }
        else {
            result = 0;
            carry = ((source_value & B_31) >> 31) != 0;
        }
        cpu.register_write(rd, result);
        // determine carry out
        arithmetic_flag_helper(cpu, true, carry, false, result);
    }
    else {
        // arithmetic right shift
        let tmp: i32 = source_value as i32;
        let result;
        let carry;
        if offset5 != 0 {
            result = tmp >> offset5;
            carry = ((source_value >> (offset5 - 1)) & B_0) != 0;
        }
        else {
            // sign extension
            result = if tmp as u32 & B_31 != 0 {0xFFFFFFFF} else {0x0};
            carry = (tmp as u32 & B_31) != 0;
        }
        cpu.register_write(rd, result as u32);
        let carry = ((source_value >> (offset5 - 1)) & B_0) != 0;
        arithmetic_flag_helper(cpu, true, carry, false, result as u32);
    }
}

pub fn add_subtract(cpu: &mut CPU, instruction: u32) {
    // p.113
    let i = (instruction as u32 & B_10 ) >> 10;
    let opcode = (instruction & B_9) >> 9;
    let rn_offset3 = (instruction & B_8_6) >> 6;
    let rs = (instruction & B_5_3) >> 3;
    let rd = instruction & B_2_0;

    let operand1 = cpu.register_read(rs);
    let operand2;
    if i == 1 {
        // rn is an immediate value
        operand2 = rn_offset3;
    }
    else {
        // rn is a register
        operand2 = cpu.register_read(rn_offset3);
    }

    if opcode == 1 {
        // subtract
        cpu.register_write(rd, operand1 - operand2);
    }
    else {
        // add
        cpu.register_write(rd, operand1 + operand2);
    }
}

pub fn move_compare_add_subtract_immediate(cpu: &mut CPU, instruction: u32) {
    // p.115
    let opcode = (instruction & B_12_11) >> 11;
    let rd = (instruction & B_10_8) >> 8;
    let offset8: u32 = instruction & B_7_0;

    if opcode == 0 {
        // immediate move
        cpu.register_write(rd, offset8);
    }
    else if opcode == 1 {
        // immediate compare
        not_implemented!()
    }
    else if opcode == 2 {
        // immediate add
        cpu.register_write(rd, cpu.register_read(rd) + offset8);
    }
    else {
        // immediate subtract
        cpu.register_write(rd, cpu.register_read(rd) - offset8);
    }
}