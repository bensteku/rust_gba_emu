use crate::{cpu::{CPUMode, ConditionFlags, Registers::*, CPU}, instructions::masks_32bit::*, not_implemented};

/*
    Logical ALU operations
*/


// there are two different flag helper functions (see below for the other) because the logical and arithmetic operations have different behavior w.r.t the V flag
#[inline]
pub fn logical_flag_helper(cpu: &mut CPU, s: bool, res: u32) {
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

pub fn tst_op(cpu: &mut CPU, _s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 & op2;
    logical_flag_helper(cpu, true, res);  // s bit always set for tst
    return 0;  // no write to Rd
}

pub fn teq_op(cpu: &mut CPU, _s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 ^ op2;
    logical_flag_helper(cpu, true, res);  // same as above
    return 0;
}

pub fn orr_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 | op2;
    logical_flag_helper(cpu, s, res);
    return res;
}

pub fn mov_op(cpu: &mut CPU, s: bool, _op1: u32, op2: u32) -> u32 {
    logical_flag_helper(cpu, s, op2);
    return op2;
}

pub fn bic_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let res = op1 & !op2;
    logical_flag_helper(cpu, s, res);
    return res;
}

pub fn mvn_op(cpu: &mut CPU, s: bool, _op1: u32, op2: u32) -> u32 {
    logical_flag_helper(cpu, s, !op2);
    return !op2;
}

/*
    Arithmetic ALU operations
*/

#[inline]
pub fn arithmetic_flag_helper(cpu: &mut CPU, s: bool, carry: bool, overflow: bool, res: u32) {
    if s {
        // z flag
        if res != 0 {cpu.set_condition_flag(ConditionFlags::Z, true);} else {cpu.set_condition_flag(ConditionFlags::Z, false);} 
        // n flag
        if res & B_31 != 0 {cpu.set_condition_flag(ConditionFlags::N, true);} else {cpu.set_condition_flag(ConditionFlags::N, false);}
        // v flag
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
    let (tmp, carry1) = op1.overflowing_add(op2);
    let (res, carry2) = tmp.overflowing_add(if cpu.get_condition_flag(ConditionFlags::C) {1} else {0});
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry1 || carry2, overflow, res);
    return res;
}

pub fn sbc_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (tmp, carry1) = op1.overflowing_sub(op2);
    let (res, carry2) = tmp.overflowing_sub(if cpu.get_condition_flag(ConditionFlags::C) {1} else {0} - 1);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry1 || carry2, overflow, res);
    return res;
}

pub fn rsc_op(cpu: &mut CPU, s: bool, op1: u32, op2: u32) -> u32 {
    let (tmp, carry1) = op2.overflowing_sub(op1);
    let (res, carry2) = tmp.overflowing_sub(if cpu.get_condition_flag(ConditionFlags::C) {1} else {0} - 1);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, s, carry1 || carry2, overflow, res);
    return res;
}

pub fn cmp_op(cpu: &mut CPU, _s: bool, op1: u32, op2: u32) -> u32 {
    let (res, carry) = op1.overflowing_sub(op2);
    let op1_sign = (op1 & B_31) != 0;
    let op2_sign = (op2 & B_31) != 0;
    let res_sign = (res & B_31) != 0;
    let overflow = (op1_sign == op2_sign) && (op1_sign != res_sign);
    arithmetic_flag_helper(cpu, true, carry, overflow, res);
    return 0;
}

pub fn cmn_op(cpu: &mut CPU, _s: bool, op1: u32, op2: u32) -> u32 {
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
    let amount = amount % 32;
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
        carry_out = (value >> amount - 1) & B_0;
        result = (value >> amount) | (value << (32 - amount));
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