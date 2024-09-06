use crate::not_implemented;

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

// lookup table for opcodes and their handling functions
// pattern, mask, handler function
type ProcFnArm = fn(u32);
type ProcFnThumb = fn(u16);
pub fn placeholder_arm(opcode: u32)
{
    not_implemented!();
}
pub fn placeholder_thumb(opcode: u16)
{
    not_implemented!();
}
const ARM_OPCODES: [(u32, u32, ProcFnArm); 15] =
    [
        (0x00000000, 0x0C000000, placeholder_arm),  // data processing
        (0x00000090, 0x0FC000F0, placeholder_arm),  // multiply
        (0x00800090, 0x0F8000F0, placeholder_arm),  // multiply long
        (0x01000090, 0x0FB00FF0, placeholder_arm),  // single data swap
        (0x013FFF10, 0x0FFFFFF0, placeholder_arm),  // branch and exchange
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
const THUMB_OPCODES: [(u16, u16, ProcFnThumb); 19] =
    [
        (0x0000, 0xE000, placeholder_thumb),  // move shifted register
        (0x1800, 0xF800, placeholder_thumb),  // add/subtract
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

// emulation of an ARMT7DMI cpu
pub struct CPU
{
    cycles: u128,
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
            cycles: 0,
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

    pub fn execute_arm(&self, instruction: u32)
    {
        let mut handled = false;
        for (pattern, mask, handler) in ARM_OPCODES
        {
            println!("{:x}", instruction);
            println!("{:x}", instruction & mask);
            println!("{:x}", pattern);
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
}