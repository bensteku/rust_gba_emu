use crate::{cpu::{CPUMode, ConditionFlags, Registers::*, CPU}, instructions::masks_32bit::*, not_implemented};

// table for opcodes and their handling functions
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

type ALUFnArm = fn(&mut CPU, bool, u32, u32) -> u32;
// DO NOT CHANGE THE ORDER HERE, IT CORRESPONDS EXACTLY TO THE ACTUAL OP CODES USED BY THE PROCESSOR
const ARM_DATA_OPS: [ALUFnArm; 16] = [
        and_op,  // AND
        eor_op,  // EOR
        sub_op,  // SUB
        rsb_op,  // RSB
        add_op,  // ADD
        adc_op,  // ADC
        sbc_op,  // SBC
        rsc_op,  // RSC
        tst_op,  // TST
        teq_op,  // TEQ
        cmp_op,  // CMP
        cmn_op,  // CMN
        orr_op,  // ORR
        mov_op,  // MOV
        bic_op,  // BIC
        mvn_op,  // MVN
];

// extra array to check for logical opcodes to handle CSPR flag effects
const ARM_DATA_OPCODES_LOGICAL: [u32; 8] = [
    0x00000000,
    0x00000001,
    0x00000008,
    0x00000009,
    0x0000000C,
    0x0000000D,
    0x0000000E,
    0x0000000F,
];

const ARM_SHIFT_TYPES: [ALUFnArm; 4] = [
    logical_left_32bit,      // 00: logical left
    logical_right_32bit,     // 01: logical right
    arithmetic_right_32bit,  // 10: arithmetic right
    rotate_32bit,            // 11: rotate right
];

pub fn process_instruction_arm(cpu: &mut CPU, instruction: u32) {
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

/*
    Instruction implementations
*/

pub fn data_processing(cpu: &mut CPU, instruction: u32) {
    // ARM manual: p. 52
    let i: bool = (instruction & B_25) != 0;
    let s: bool = (instruction & B_20) != 0;
    let rn: u32 = (instruction & B_19_16) >> 16;
    let rd: u32 = (instruction & B_15_12) >> 12;
    //let op2: u32 = instruction & B_11_0;
    let opcode: u32 = (instruction & B_24_21) >> 21;

    // resolve first operand
    let op1 = cpu.register_read(rn);
    // resolve the second operand
    let op2;
    if i {
        // I flag is set, meaning the second operand is an immediate value
        // procedure: extend the 8 bit value to 32 bit, then rotate by twice the amount in the rotate bits
        let rotate: u32 = (instruction & B_11_8) >> 8;
        if rotate != 0 {
            op2 = rotate_32bit(cpu, s, instruction & B_7_0, rotate * 2);
        }
        else {
            op2 = instruction & B_7_0;
        }
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
            // in this case, we load in the shift amount from the bottom byte of the register mentioned in bits 11 to 8
            let shift_register_address: u32 = (instruction & B_11_8) >> 8;
            let shift_register_value: u32 = cpu.register_read(shift_register_address.try_into().unwrap());
            shift_amount = shift_register_value & B_7_0;
        }
        else {
           // in this case the amount is determined from the instruction
           shift_amount = (instruction & B_11_7) >> 7; 
        }
        let shift_type: u32 = (instruction & B_6_5) >> 5;
        // note: we run the shifts even if the shift amount turns out to be 0 such that the carry flags get affected correctly
        // even if nothing actually happens to the operand value
        op2 = ARM_SHIFT_TYPES[shift_type as usize](cpu, s, op2_init_value, shift_amount);    
    }
    // now that both operands are known, we can apply the operations onto it
    let res = ARM_DATA_OPS[opcode as usize](cpu, s, op1, op2);
    // write result
    cpu.register_write(rd, res);
    // if s is set and we write to R15, we need to copy over the SPSR into the CPSR
    if s && rd == 15 {
        if cpu.get_mode() == CPUMode::User {
            panic!("Trying to write into R15 with s bit set while in User mode!")
        }
        cpu.register_write(16, cpu.register_read(17));
    }
}

pub fn branch_and_exchange(cpu: &mut CPU, instruction: u32) {
    // ARM manual: p. 48
    // handle registers in other CPU modes
    let rn: u32 = instruction & B_3_0;
    let t_bit: u32 = instruction & B_0;
    if rn == 15 {
        println!("[WARNING] Branch and exchange instruction into the program counter register (R15), undefined behavior!")
    }
    cpu.registers[R15] = cpu.register_read(rn);
    cpu.t = t_bit != 0;
}

pub fn branch(cpu: &mut CPU, instruction: u32) {
    // ARM manual: p. 50
    let bit_24: u32 = instruction & B_24;
    if bit_24 != 0 {
        // link bit is set, copy old PC into R14
        cpu.register_write(14, cpu.registers[R15]);
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

/*
    Logical ALU operations
*/

#[inline]
fn logical_flag_helper(cpu: &mut CPU, s: bool, res: u32) {
    if s {
        if res == 0 {
            cpu.set_condition_flag(ConditionFlags::Z, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::Z, false);
        }
        if res & B_31 != 0 {
            cpu.set_condition_flag(ConditionFlags::N, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::N, false);
        }
    }
}

pub fn and_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 & op2;
    logical_flag_helper(cpu, s, res);
    return res;
}

pub fn eor_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 ^ op2;
    logical_flag_helper(cpu, s, res);
    return res;
}

pub fn tst_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 & op2;
    logical_flag_helper(cpu, true, res);  // s bit always set for tst
    return 0;  // no write to Rd
}

pub fn teq_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 ^ op2;
    logical_flag_helper(cpu, true, res);  // same as above
    return 0;
}

pub fn orr_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 | op2;
    logical_flag_helper(cpu, s, res);
    return res;
}

pub fn mov_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    logical_flag_helper(cpu, s, op2);
    return op2;
}

pub fn bic_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 & !op2;
    logical_flag_helper(cpu, s, res);
    return res;
}

pub fn mvn_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    logical_flag_helper(cpu, s, !op2);
    return !op2;
}

/*
    Arithmetic ALU operations
*/

#[inline]
fn arithmetic_flag_helper(cpu: &mut CPU, s: bool, carry: bool, overflow: bool, res: u32) {
    if s {
        // z flag
        if res != 0 {cpu.set_condition_flag(ConditionFlags::Z, true);} else {cpu.set_condition_flag(ConditionFlags::Z, false);} 
        // n flag
        if res & B_31 != 0 {cpu.set_condition_flag(ConditionFlags::N, true);} else {cpu.set_condition_flag(ConditionFlags::N, false);}
        // c flag
        if overflow {cpu.set_condition_flag(ConditionFlags::V, true);} else {cpu.set_condition_flag(ConditionFlags::V, false);}
        // c flag
        if carry {cpu.set_condition_flag(ConditionFlags::C, true);} else {cpu.set_condition_flag(ConditionFlags::C, false);}
    }
}

pub fn sub_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (res, carry) = op1.overflowing_sub(op2);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry, overflow, res);
    return res;
}

pub fn rsb_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (res, carry) = op2.overflowing_sub(op1);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry, overflow, res);
    return res;
}

pub fn add_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (res, carry) = op1.overflowing_add(op2);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry, overflow, res);
    return res;
}

pub fn adc_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (mut res, carry) = op1.overflowing_add(op2);
    res += if cpu.get_condition_flag(ConditionFlags::C) {1} else {0};
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry, overflow, res);
    return res;
}

pub fn sbc_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (mut res, carry) = op1.overflowing_sub(op2);
    res += if cpu.get_condition_flag(ConditionFlags::C) {1} else {0} - 1;
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry, overflow, res);
    return res;
}

pub fn rsc_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (mut res, carry) = op2.overflowing_sub(op1);
    res += if cpu.get_condition_flag(ConditionFlags::C) {1} else {0} - 1;
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry, overflow, res);
    return res;
}

pub fn cmp_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (res, carry) = op1.overflowing_sub(op2);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, true, carry, overflow, res);
    return 0;
}

pub fn cmn_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (res, carry) = op1.overflowing_add(op2);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, true, carry, overflow, res);
    return 0;
}

/*
    Barrel Shifter Functions
*/

pub fn rotate_32bit(cpu: &mut CPU, s: bool, value: u32, amount: u32) -> u32 {
    // rotates a 32 bit binary value to the right

    // note: in this and all follow up functions the carry out is strictly not the bit itself
    // but a u32 with all zeros outside of the bit where the information is extracted from
    // a comparison with 0 then determines the bit's value without any shifting
    let carry_out: u32;
    let result: u32;

    // amount 0 encodes rotate right extended:
    // carry out is bit 0 of the input,
    // output is created by rotating input once, then replacing the highest bit with the C flag
    if amount == 0 {
        carry_out = value & B_0;
        if cpu.get_condition_flag(ConditionFlags::C) {
            result = ((value >> 1) | (value << (32 - 1))) | (1 << 31);
        }
        else {
            result = ((value >> 1) | (value << (32 - 1))) & !(1 << 31);
        }
        
    }
    else if amount == 32 {
        carry_out = value & B_31;
        result = value;
    }
    else {
        let rotate = amount % 32;
        carry_out = (value >> rotate - 1) & B_0;
        result = (value >> rotate) | (value << (32 - rotate));
    }
    
    if s {
        if carry_out != 0 {
            cpu.set_condition_flag(ConditionFlags::C, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::C, false);
        }
    }

    return result;
}

pub fn logical_left_32bit(cpu: &mut CPU, s: bool, value: u32, amount: u32) -> u32 {
    // logically shifts a 32 bit binary value leftwards

    let carry_out: u32;
    let result: u32;

    // in case the amount is 32, apply the special rule from the ARM instruction manual:
    // zero result, carry out is first bit of input
    if amount == 32 {
        carry_out = B_0 & value;
        result = 0;
    }
    // the next case:
    // zero result, carry out zero
    else if amount > 32 {
        carry_out = 0;
        result = 0;
    }
    // in other cases do the shift as normal
    // carry out is the leftmost bit of the original value
    else {
        let one_less_shift = value << (amount - 1);
        carry_out = one_less_shift & B_31;
        result = one_less_shift << 1;
    }
    // handle carry out
    // note: if amount == 0, no carry out change is supposed to propagate
    if s && amount != 0 {
        if carry_out != 0 {
            cpu.set_condition_flag(ConditionFlags::C, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::C, false);
        }
    }
    return result;
}

pub fn logical_right_32bit(cpu: &mut CPU, s: bool, value: u32, amount: u32) -> u32 {
    // logically shifts a 32 bit binary value rightwards

    let carry_out: u32;
    let result: u32;

    // amount 0 encodes LSR 32, which means zero output and bit 31 of input as the carry out
    if amount == 0 || amount == 32 {
        carry_out = value & B_31;
        result = 0;
    }
    else if amount > 32 {
        carry_out = 0;
        result = 0;
    }
    else {
        let one_less_shift = value >> (amount - 1);
        carry_out = one_less_shift & B_0;
        result = one_less_shift >> 1;
    }

    if s {
        if carry_out != 0 {
            cpu.set_condition_flag(ConditionFlags::C, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::C, false);
        }
    }

    return result;
}

pub fn arithmetic_right_32bit(cpu: &mut CPU, s: bool, value: u32, amount: u32) -> u32 {
    // shifts a 32 bit binary value rightwards, filling up vacated places with the value of bit 31 of the input

    let carry_out: u32;
    let result: u32;

    // pass through to left shift if amount is 0
    if amount == 0 {
        return logical_left_32bit(cpu, s, value, 0);
    }
    // if amount is 32 or greater fill output and carry bit with bit 31 of the input
    else if amount >= 32 {
        carry_out = value & B_31;
        result = if carry_out != 0 { 0xFFFFFFFF } else { 0x0 };
    }
    else {
        // in Rust, signed types apparently right shift arithmetically inherently
        // so instead of implementing this myself, I'll just convert and reconvert
        // still have to get the carry bit though
        let temp = value as i32;
        let one_less_shift = value >> (amount - 1);
        carry_out = one_less_shift & B_0;
        result = (temp >> amount) as u32;
    }
    if s {
        if carry_out != 0 {
            cpu.set_condition_flag(ConditionFlags::C, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::C, false);
        }
    }

    return result;
}