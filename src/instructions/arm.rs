use std::collections::HashSet;
use crate::{cpu::{CPUMode, ConditionFlags, Registers::*, CPU}, 
            instructions::masks_32bit::*, 
            instructions::basic_ops::*,
            not_implemented};

// table for opcodes and their handling functions
// pattern, mask, handler function
type ProcFnArm = fn(&mut CPU, u32);
pub fn placeholder_arm(cpu: &mut CPU, opcode: u32) {
    not_implemented!();
}
const ARM_OPCODES: [(u32, u32, ProcFnArm); 17] = [
        (0x00000000, 0x0C000000, data_processing),  // data processing
        (0x010F0000, 0x0FBF0FFF, mrs),  // MRS
        (0x0129F000, 0x0FBFFFF0, msr_full),  // MSR
        (0x0128F000, 0x0DBFF000, msr_flags),  // MSR (flag bits only)
        (0x00000090, 0x0FC000F0, multiply),  // multiply
        (0x00800090, 0x0F8000F0, multiply_long),  // multiply long
        (0x01000090, 0x0FB00FF0, single_data_swap),  // single data swap
        (0x013FFF10, 0x0FFFFFF0, branch_and_exchange),  // branch and exchange
        (0x00000090, 0x0E000090, halfword_signed_data_transfer),  // halfword data transfer
        (0x04000000, 0x0C000000, single_data_transfer),  // single data transfer
        (0x06000010, 0x0E000010, placeholder_arm),  // undefined
        (0x08000000, 0x0E000000, block_data_transfer),  // block data transfer
        (0x0A000000, 0x0E000000, branch),  // branch
        (0x0C000000, 0x0E000000, placeholder_arm),  // coprocessor data transfer
        (0x0E000000, 0x0F000000, placeholder_arm),  // coprocessor data operation
        (0x0E000010, 0x0F000010, placeholder_arm),  // coprocessor register transfer
        (0x0F000000, 0x0F000000, software_interrupt),  // software interrupt
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

// helper function to check whether an op is logical or arithmetical
// should compile to a O(1) check
fn logical_op(op: u32) -> bool {
    match op {
        0 | 1 | 8 | 9 | 12 | 13 | 14 | 15 => true,
        _ => false
    }
}


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
    let opcode: u32 = (instruction & B_24_21) >> 21;

    // check whether the operation we're supposed to perform is either logical or arithmetic
    // in case it's logical, we'll use the carry out from the shifting operations, otherwise we ignore it
    let logical: bool = logical_op(opcode);
    
    // resolve first operand
    let op1 = cpu.register_read(rn);
    // resolve the second operand
    let op2;
    if i {
        // I flag is set, meaning the second operand is an immediate value
        // procedure: extend the 8 bit value to 32 bit, then rotate by twice the amount in the rotate bits
        let rotate: u32 = (instruction & B_11_8) >> 8;
        if rotate != 0 {
            op2 = rotate_32bit(cpu, logical, instruction & B_7_0, rotate * 2);
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

        let shift_amount  = get_shift_amount(cpu, instruction);
        // this is now wrapped in the helper function in the line as the same procedure is used with other instructions as well
        /*
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
        */
        let shift_type: u32 = (instruction & B_6_5) >> 5;
        // note: we run the shifts even if the shift amount turns out to be 0 such that the carry flags get affected correctly
        // even if nothing actually happens to the operand value
        op2 = ARM_SHIFT_TYPES[shift_type as usize](cpu, logical, op2_init_value, shift_amount);    
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

pub fn mrs(cpu: &mut CPU, instruction: u32) {
    // ARM manual p. 61
    let rd = (instruction & B_15_12) >> 12;
    // if bit is set SPSR, else CPSR
    let source_psr;
    if (instruction & B_22) != 0 {
        source_psr = 17;
        if cpu.get_mode() == CPUMode::User {
            panic!("Access to SPSR in User mode!")
        }
    } 
    else {
        source_psr = 16;
    }
    cpu.register_write(rd, cpu.register_read(source_psr));
}

pub fn msr_full(cpu: &mut CPU, instruction: u32) {
    // ARM manual p. 61
    let rm = instruction & B_3_0;
    let dest_psr;
    if (instruction & B_22) != 0 {
        dest_psr = 17;
        if cpu.get_mode() == CPUMode::User {
            panic!("Access to SPSR in User mode!")
        }
    } 
    else {
        dest_psr = 16;
    }
    cpu.register_write(dest_psr, cpu.register_read(rm));
}

pub fn msr_flags(cpu: &mut CPU, instruction: u32) {
    // ARM manual p. 61
    let i = (instruction & B_25) != 0;
    let op;
    if i {
        let rotate = (instruction & B_11_8) >> 8;
        let immediate = instruction & B_7_0;
        op = rotate_32bit(cpu, false, immediate, rotate);
    }
    else {
        op = cpu.register_read(instruction & B_3_0);
    }
    let dest_psr;
    if (instruction & B_22) != 0 {
        dest_psr = 17;
        if cpu.get_mode() == CPUMode::User {
            panic!("Access to SPSR in User mode!")
        }
    } 
    else {
        dest_psr = 16;
    }
    // for this method, only the top four bits are written, i.e. the flag bits
    //      get current psr and null top four bits
    let nulled_psr = cpu.register_read(dest_psr) & 0x0FFFFFFF;
    //      or with top four bits of new value and write to destination
    cpu.register_write(dest_psr, nulled_psr | (op & 0xF000000) );
}

pub fn branch_and_exchange(cpu: &mut CPU, instruction: u32) {
    // ARM manual: p. 48
    // handle registers in other CPU modes
    let rn: u32 = instruction & B_3_0;
    let t_bit: u32 = instruction & B_0;
    if rn == 15 {
        println!("[WARNING] Branch and exchange instruction into the program counter register (R15), undefined behavior!")
    }
    cpu.registers[R15] = cpu.register_read(rn) & !0b11;  // null out last two bits
    cpu.set_state(t_bit != 0);
    cpu.branch = true;
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
    cpu.branch = true;
}

pub fn multiply(cpu: &mut CPU, instruction: u32) {
    // ARM manual p. 65
    let accumulate = (instruction & B_21) != 0;
    let s = (instruction & B_20) != 0;

    let rd = (instruction & B_19_16) >> 16;
    let rn = (instruction & B_15_12) >> 12;
    let rs = (instruction & B_11_8) >> 8;
    let rm = instruction & B_3_0;

    // operand restrictions
    if rd == rm {
        panic!("Multiply operand Rd ({}) must not be the same as operand Rm ({})!", rd, rm);
    }
    else if rd == 15 || rm == 15 || rs == 15 || rn == 15 {
        panic!("Register R15 must not be used in multiply operations!")
    }

    let res;
    if accumulate {
        res = cpu.register_read(rm).wrapping_mul(cpu.register_read(rs)).wrapping_add(cpu.register_read(rn));
    }
    else {
        res = cpu.register_read(rm).wrapping_mul(cpu.register_read(rs));
    }

    logical_flag_helper(cpu, s, res);
    cpu.register_write(rd, res);
}

pub fn multiply_long(cpu: &mut CPU, instruction: u32) {
    // ARM manual p. 67
    let unsigned = (instruction & B_22) != 0;
    let accumulate = (instruction & B_21) != 0;
    let s = (instruction & B_20) != 0;

    let rd_hi = (instruction & B_19_16) >> 16;
    let rd_lo = (instruction & B_15_12) >> 12;
    let rs = (instruction & B_11_8) >> 8;
    let rm = instruction & B_3_0;

    // operand restrictions
    if rd_hi == rd_lo || rd_hi == rm || rd_lo == rm {
        panic!("Multiply long operands Rd_hi ({}), Rd_lo ({}) and Rm ({}) must all be distinct from each other!", rd_hi, rd_lo, rm);
    }
    else if rd_hi == 15 || rd_lo == 15 || rs == 15 || rm == 15 {
        panic!("Register R15 must not be used in multiply long operations!")
    }

    let res_hi;
    let res_lo;
    if accumulate {
        if unsigned {
            let prod = (cpu.register_read(rm) as u64) * (cpu.register_read(rs) as u64);
            let add = (cpu.register_read(rd_hi) as u64) << 32 + (cpu.register_read(rd_lo) as u64);
            let prod1 = prod + add;
            res_lo = prod1 as u32;
            res_hi = (prod1 >> 32) as u32;
        }
        else {
            let prod = (cpu.register_read(rm) as i64) * (cpu.register_read(rs) as i64);
            let add = (cpu.register_read(rd_hi) as i64) << 32 + (cpu.register_read(rd_lo) as i64);
            let prod1 = prod + add;
            res_lo = prod1 as u32;
            res_hi = (prod1 >> 32) as u32;
        }
    }
    else {
        if unsigned {
            let prod = (cpu.register_read(rm) as u64) * (cpu.register_read(rs) as u64);
            res_lo = prod as u32;
            res_hi = (prod >> 32) as u32;
        }
        else {
            let prod = (cpu.register_read(rm) as i64) * (cpu.register_read(rs) as i64);
            res_lo = prod as u32;
            res_hi = (prod >> 32) as u32;
        }
    }

    if s {
        if res_lo == 0 && res_hi == 0 {
            cpu.set_condition_flag(ConditionFlags::Z, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::Z, false);
        }
        if res_hi & B_31 != 0 {
            cpu.set_condition_flag(ConditionFlags::N, true);
        }
        else {
            cpu.set_condition_flag(ConditionFlags::N, false);
        }
    }

    cpu.register_write(rd_hi, res_hi);
    cpu.register_write(rd_lo, res_lo);

}

pub fn single_data_transfer(cpu: &mut CPU, instruction: u32) {
    // ARM manual p. 70
    let i = (instruction & B_25) != 0;
    let p = (instruction & B_24) != 0;
    let u = (instruction & B_23) != 0;
    let b = (instruction & B_22) != 0;
    let w = (instruction & B_21) != 0;
    let l = (instruction & B_20) != 0;

    let rn = (instruction & B_19_16) >> 16;
    let rd = (instruction & B_15_12) >> 12;
    let base_address = cpu.register_read(rn);
    
    // guards against using R15
    if rn == 15 && w {
        panic!("Must not use R15 with write-back in Single Data Transfer instruction!")
    }

    // determine offset
    let offset;
    if i {
        let rm = instruction & B_3_0;
        if rm == 15 {
            panic!("Rm must not be R15 in Single Data Transfer instruction!")
        }
        let offset_init_value = cpu.register_read(rm);

        let shift_amount = get_shift_amount(cpu, instruction);
        let shift_type: u32 = (instruction & B_6_5) >> 5;
        offset = ARM_SHIFT_TYPES[shift_type as usize](cpu, false, offset_init_value, shift_amount);
    }
    else {
        offset = instruction & B_11_0;
    }
    // calculate offset adress
    let offset_address;
    if u {
        offset_address = base_address + offset;
    }
    else {
        offset_address = base_address - offset;
    }
    // write back offset address if so desired
    // TODO: look at the w bit in privileged mode
    if w || !p {
        cpu.register_write(rn, offset_address);
    }
    
    // perform memory transfer
    if l {
        let load_value = cpu.memory_read(offset_address, if b {0} else {2});
        cpu.register_write(rd, load_value);
    }
    else {
        let store_value = cpu.register_read(rd);
        cpu.memory_write(offset_address, if b {0} else {2}, store_value);
    }

}

pub fn halfword_signed_data_transfer(cpu: &mut CPU, instruction: u32) {
    let l = (instruction & B_20) != 0;
    let w = (instruction & B_21) != 0;
    let i = (instruction & B_22) != 0;
    let u = (instruction & B_23) != 0;
    let p = (instruction & B_24) != 0;
    let s = (instruction & B_6) != 0;
    let h = (instruction & B_5) != 0;

    if s && l {
        panic!("In halfword/signed data transfer instruction {:b} the S and L bits are both set!", instruction);
    }
    if !s && !h {
        panic!("Swap demanded in halfword/signed data transfer instruction {:b}!", instruction)
    }

    let rn = (instruction & B_19_16) >> 16;
    let rd = (instruction & B_15_12) >> 12;
    
    let base_address = cpu.register_read(rn);

    // determine offset
    let offset;
    if i {
        // immediate offset
        let offset_lower = instruction & B_3_0;
        let offset_upper = instruction & B_11_8 >> 4;
        offset = offset_lower + offset_upper;
    }
    else {
        // offset from register
        let rm = instruction & B_3_0;
        offset = cpu.register_read(rm);
    }
    
    // calculate offset address
    let offset_address;
    if u {
        offset_address = base_address + offset;
    }
    else {
        offset_address = base_address - offset;
    }
    // write back offset address if so desired
    if w || !p {
        cpu.register_write(rn, offset_address);
    }

    // load/store
    if l {
        let load_data;
        if s {
            if h {
                // signed halfword load
                load_data = cpu.memory_read(offset_address, 1);
                let tmp: u16 = load_data as u16;
                let tmp: i16 = tmp as i16;
                let tmp: i32 = tmp as i32;
                cpu.register_write(rd, tmp as u32);
            }
            else {
                // signed byte load
                load_data = cpu.memory_read(offset_address, 0);
                // sign extend to all bits
                let tmp: u8 = load_data as u8;  // works because memory read places the byte into lower 8 bits
                let tmp: i8 = tmp as i8;
                let tmp: i32 = tmp as i32;  // sign extension done automatically by Rust
                cpu.register_write(rd, tmp as u32);
            }
        }
        else {
            // no need to deal with h, would be a swap, that's dealt with separately
            // top 16 bits have to be set to 0, this is done in the memory read already
            load_data = cpu.memory_read(offset_address, 1);
            cpu.register_write(rd, load_data);
        }
    }
    else {
        // here we don't need to ask for h or s, since only halfword stores are possible
        // if the flags aren't set for that, the function should already have paniced above
        cpu.memory_write(offset_address, 1, cpu.register_read(rd) & B_15_0);
    }

}

pub fn single_data_swap(cpu: &mut CPU, instruction: u32) {
    // p.89
    let b = if (instruction & B_22) != 0 {0} else {2};

    let rn = (instruction & B_19_16) >> 16;
    let rd = (instruction & B_15_12) >> 12;
    let rm = instruction & B_3_0;

    // R15 block
    if rn == 15 || rd == 15 || rm == 15 {
        panic!("Swap with rn {}, rd {}, rm {}, involves R15 which is not allowed.", rn, rd, rm);
    }

    // read word or byte from base address
    let memory_read = cpu.memory_read(rn, b);
    // write swap register into memory
    cpu.memory_write(rn, b, cpu.register_read(rm));
    // overwrite swap register
    cpu.register_write(rd, memory_read);
}

pub fn block_data_transfer(cpu: &mut CPU, instruction: u32) {
    // p.82
    let p = (instruction & B_24) != 0;
    let u = (instruction & B_23) != 0;
    let s: bool = (instruction & B_22) != 0;
    let w = (instruction & B_21) != 0;
    let l = (instruction & B_20) != 0;

    let rn = (instruction & B_19_16) >> 16;
    let register_list = instruction & B_15_0;

    // R15 block
    if rn == 15 {
        panic!("R15 used as base register in Block Data Transfer instruction {:b}!", instruction);
    }

    let base_address = cpu.register_read(rn);
    let mut cur_address = base_address;
    if !u {
        cur_address -= register_list.count_ones();
        if p {
            cur_address -= 1;  // correct for the effects of pre-decrement, see graphics on p.85 of the PDF
        }
        else {
            cur_address += 1;  // same as above, just for post-decrement
        }
    }

    // check if R15 is present and s is set, we need to change a few things
    let mut usermode_switch= false;
    if s {
        if l {
            if register_list & B_15 != 0 {
                // SPSR of mode is transferred to CPSR if R15 in list and s is set
                cpu.register_write(16, cpu.register_read(17));
            }
            else {
                usermode_switch = true;
            }   
        }
        else {
            usermode_switch = true;
        }
    }

    for i in 0..15 {
        if register_list & (1 << i) == 0 {
            continue;
        }
        // pre
        if p {
            cur_address += 1;
        }
        if l {
            // load
            if usermode_switch {
                cpu.register_write_custom(i, cpu.memory_read(cur_address, 2), CPUMode::User);
            }
            else {
                cpu.register_write(i, cpu.memory_read(cur_address, 2));
            }
        }
        else {
            // store
            if usermode_switch {
                // memory write with the registers being read from User mode instead of current mode
                cpu.memory_write(cur_address, 2, cpu.register_read_custom(i, CPUMode::User));
            }
            else {
                cpu.memory_write(cur_address, 2, cpu.register_read(i));
            }
        }
        // post
        if !p {
            cur_address += 1;
        }
    }

    // write back modified address
    if w {
        cpu.register_write(rn, if u {base_address + register_list.count_ones()} else {base_address - register_list.count_ones()});
    }
}

pub fn software_interrupt(cpu: &mut CPU, _instruction: u32) {
    // change mode to software interrupt
    cpu.set_mode(CPUMode::Supervisor);
    // save PC in R14_svc
    cpu.register_write(14, cpu.register_read(15));
    // set PC to 0x08
    cpu.register_write(15, 0x08);
    // save CPSR to SPSR_svc
    cpu.register_write(17, cpu.register_read(16));
    // most likely unfinished
}

pub fn coprocessor_data_operations(cpu: &mut CPU, instruction: u32) {
    // p.93
    let cp_opc = (instruction & B_23_20) >> 20;
    let crn = (instruction & B_19_16) >> 16;
    let crd = (instruction & B_15_12) >> 12;
    let cphash = (instruction & B_11_8) >> 8;
    let cp = (instruction & B_7_5) >> 5;
    let crm = (instruction & B_3_0);

    // nothing else to do here apparently 
}

pub fn coprocessor_data_transfer(cpu: &mut CPU, instruction: u32) {
    // p.95
    let p = (instruction & B_24) != 0;
    let u = (instruction & B_23) != 0;
    let n = (instruction & B_22) != 0;
    let w = (instruction & B_21) != 0;
    let l = (instruction & B_20) != 0;

    let rn = (instruction & B_19_16) >> 16;
    let crd = (instruction & B_15_12) >> 12;
    let cphash = (instruction & B_11_8) >> 8;
    let offset = instruction & B_7_0;

    if w && rn == 15 {
        panic!("R15 must not be the base register in coprocessor data transfer with write back enabled, instruction: {:b}.", instruction);
    }

    not_implemented!();
}

pub fn coprocessor_register_transfer(cpu: &mut CPU, instruction: u32) {
    not_implemented!();
}

pub fn undefined(cpu: &mut CPU, instruction: u32) {
    not_implemented!();
}

// little helper to recycle code for shift by register/immediate shift
#[inline]
fn get_shift_amount(cpu: &mut CPU, instruction: u32) -> u32 {
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
    return shift_amount;
}