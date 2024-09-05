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
    let mut _cpu = CPU::new();
    cart.info();
    for i in 0..=100
    {
        println!("{:08b}", cart.read_adress(i));
    }
    
    Ok(())
}
