use std::fs::{read, File};
use std::io::Read;

pub struct CartridgeHeader
{
    entry: [u8; 4],
    logo: [u8; 156],
    pub title: [u8; 12],
    game_code: [u8; 4],
    maker_code: [u8; 2],
    fixed_value: u8,
    main_unit_code: u8,
    device_type: u8,
    reserve_area: [u8; 7],
    software_version: u8,
    complement_check: u8,
    reserved_area: [u8; 2],
    ram_entry_point: [u8; 4],
    boot_mode: u8,
    slave_id_number: u8,
    not_used: [u8; 26],
    joybus_entry_point: [u8; 4],
}

pub struct Cartridge 
{
    pub header: CartridgeHeader,
    rom_data: Vec<u8>,
}

fn read_rom(filename: String) -> std::io::Result<Vec<u8>> 
{
    let mut file = File::open(filename)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn create_cartridge_header(bytes: &[u8]) -> CartridgeHeader
{
    CartridgeHeader
    {
        entry: 
        {
            let mut tmp = [0u8; 4];
            tmp.copy_from_slice(&bytes[0..4]);
            tmp
        },
        logo:
        {
            let mut tmp = [0u8; 156];
            tmp.copy_from_slice(&bytes[4..160]);
            tmp 
        },
        title:
        {
            let mut tmp = [0u8; 12];
            tmp.copy_from_slice(&bytes[160..172]);
            tmp 
        },
        game_code:
        {
            let mut tmp = [0u8; 4];
            tmp.copy_from_slice(&bytes[172..176]);
            tmp 
        },
        maker_code:
        {
            let mut tmp = [0u8; 2];
            tmp.copy_from_slice(&bytes[176..178]);
            tmp 
        },
        fixed_value: bytes[178],
        main_unit_code: bytes[179],
        device_type: bytes[180],
        reserve_area:
        {
            let mut tmp = [0u8; 7];
            tmp.copy_from_slice(&bytes[181..188]);
            tmp 
        },
        software_version: bytes[188],
        complement_check: bytes[189],
        reserved_area:
        {
            let mut tmp = [0u8; 2];
            tmp.copy_from_slice(&bytes[190..192]);
            tmp 
        },
        ram_entry_point:
        {
            let mut tmp = [0u8; 4];
            tmp.copy_from_slice(&bytes[192..196]);
            tmp 
        },
        boot_mode: bytes[196],
        slave_id_number: bytes[197],
        not_used:
        {
            let mut tmp = [0u8; 26];
            tmp.copy_from_slice(&bytes[198..224]);
            tmp 
        },
        joybus_entry_point:
        {
            let mut tmp = [0u8; 4];
            tmp.copy_from_slice(&bytes[224..]);
            tmp 
        },
    }
}

pub fn create_cartridge(filename: String) -> std::io::Result<Cartridge> 
{
    let bytes = read_rom(filename)?;
    let ret = Cartridge 
    {
        header: create_cartridge_header(&bytes[0..228]),
        rom_data: 
        {
            let mut tmp = vec![0u8; bytes.len() - 228];
            tmp.copy_from_slice(&bytes[228..]);
            tmp
        },
    };
    Ok(ret)
}