use std::env;
use std::error::Error;

pub mod cartridge;

fn main() -> Result<(), Box<dyn Error>>
{
    let args: Vec<String> = env::args().collect();
    let filename = args[1].clone();
    let cart = cartridge::create_cartridge(filename)?;
    let string = String::from_utf8_lossy(&cart.header.title);
    println!("{}", string);
    Ok(())
}
