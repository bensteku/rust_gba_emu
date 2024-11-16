// various mask to extract certain bits from the instruction
pub const B_3_0:   u32 = 0x0000000F;  // lowest 4 bits
pub const B_4_0:   u32 = 0x0000001F;  // lowest 5 bits
pub const B_0:     u32 = 0x00000001;  // lowest bit
pub const B_24:    u32 = 0x01000000;  // bit 24
pub const B_22:    u32 = 0x00400000;  // bit 22
pub const B_21:    u32 = 0x00200000;  // bit 21
pub const B_23:    u32 = 0x00800000;  // bit 23
pub const B_23_0:  u32 = 0x00FFFFFF;  // lower 24 bits
pub const B_25:    u32 = 0x02000000;  // bit 25
pub const B_20:    u32 = 0x00100000;  // bit 20
pub const B_24_21: u32 = 0x03E00000;  // bits 24 to 21
pub const B_19_16: u32 = 0x000F0000;  // bits 19 to 16
pub const B_15_12: u32 = 0x0000F000;  // bits 15 to 12
pub const B_11_0:  u32 = 0x00000FFF;  // lowest 12 bits
pub const B_3:     u32 = 0x00000008;  // bit 3
pub const B_4:     u32 = 0x00000010;  // bit 4
pub const B_5:     u32 = 0x00000020;  // bit 5
pub const B_6:     u32 = 0x00000040;  // bit 6
pub const B_7:     u32 = 0x00000040;  // bit 7
pub const B_6_5:   u32 = 0x00000060;  // bits 6 and 5
pub const B_11_7:  u32 = 0x00000780;  // bits 11 to 7
pub const B_11_8:  u32 = 0x00000700;  // bits 11 to 8
pub const B_11_4:  u32 = 0x00000FF0;  // bits 11 to 4
pub const B_7_0:   u32 = 0x000000FF;  // bits 7 to 0
pub const B_31:    u32 = 0x80000000;  // bit 31
pub const B_31_28: u32 = 0xF0000000;  // bits 31 to 28
pub const B_15_0:  u32 = 0x0000FFFF;  // first 16 bits
pub const B_15:    u32 = 0x00008000;  // bit 15