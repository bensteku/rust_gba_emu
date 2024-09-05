use crate::{macros::*, not_implemented};

#[derive(Debug, Default)]
pub struct Registers
{
   // regular registers
   r0: u32,
   r1: u32, 
   r2: u32, 
   r3: u32, 
   r4: u32, 
   r5: u32, 
   r6: u32, 
   r7: u32, 
   // r8 to r12 are not availalbe in THUMB mode
   r8: u32, 
   r9: u32, 
   r10: u32, 
   r11: u32, 
   r12: u32, 
   // stack pointer
   r13: u32,
   // subroutine link register
   r14: u32, 
   // program counter register
   r15: u32,  // in ARM mode: bits 31:2 -> PC, bits 1:0 -> 0, in THUMB mode: 31:1 -> PC, bit 0 -> 0
   // banked registers
   //   fast interrupt request mode
   r8_fiq: u32, 
   r9_fiq: u32, 
   r10_fiq: u32, 
   r11_fiq: u32, 
   r12_fiq: u32, 
   r13_fiq: u32, 
   r14_fiq: u32, 
   //   interrupt request mode
   r13_irq: u32, 
   r14_irq: u32, 
   //   supervisor 
   r13_svc: u32, 
   r14_svc: u32,
   //   abort 
   r13_abt: u32, 
   r14_abt: u32,
   //   undefined 
   r13_und: u32, 
   r14_und: u32,
   // status registers 
   cpsr: u32, 
   spsr_fiq: u32, 
   spsr_svc: u32, 
   spsr_abt: u32, 
   spsr_irq: u32, 
   spsr_und: u32, 
}

enum CPUMode
{
    User,
    FIQ,
    IRQ,
    Supervisor,
    Abort,
    Undefined,
    System,
}

// emulation of an ARMT7DMI cpu
pub struct CPU
{
    registers: Registers,
    t: bool,  // true for THUMB mode, false for ARM mode
    mode: CPUMode,
    n: bool,  // negative flag
    z: bool,  // zero flag
    c: bool,  // carry/borrow/extend flag
    v: bool,  // overflow flag
}

impl CPU
{
    pub fn new() -> CPU
    {
        return CPU {
            registers: {
                let mut reg: Registers = Default::default();
                reg
            },
            t: false,
            mode: CPUMode::User,
            n: false,
            z: false,
            c: false,
            v: false,
        }
    }

    pub fn execute(&self, instruction: u32) -> u32
    {
        if self.t
        {
            not_implemented!();
        }
        else
        {
            return 0;
        }
    }
}