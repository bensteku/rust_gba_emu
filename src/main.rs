use std::env;
use std::error::Error;
use cartridge::Cartridge;
use cpu::CPU;

pub mod cartridge;
pub mod cpu;
pub mod macros;

fn main() -> Result<(), Box<dyn Error>>
{
    let args: Vec<String> = env::args().collect();
    let filename = args[1].clone();
    let cart = Cartridge::new(filename)?;
    let mut cpu = CPU::new();
    cart.info();
    for i in 0..=100
    {
        println!("{:08b}", cart.read_adress(i));
    }
    let test1: u32 = 0b00000000111100101000000001101001;
    let test2: u32 = 0b00001000111100101000000001101001;

    //cpu.execute_arm(test1);
    cpu.execute_arm(test2);

    Ok(())
}
