use crate::not_implemented;
type ProcFnThumb = fn(u16);
pub fn placeholder_thumb(opcode: u16)
{
    not_implemented!();
}
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