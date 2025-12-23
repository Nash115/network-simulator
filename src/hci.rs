use std::io::{self, Write};

use crate::ip::IP;
use crate::mac::MAC;


pub enum HciError {
    ErrorReadingStdin,
    ErrorParsingInput,
    InvalidDataFormat
}

impl std::fmt::Display for HciError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HciError::ErrorReadingStdin => write!(f, "Error Reading Stdin"),
            HciError::ErrorParsingInput => write!(f, "Error Parsing Input"),
            HciError::InvalidDataFormat => write!(f, "Invalid Data Format")
        }
    }
}

pub fn get_input(prompt: &str) -> Result<String, HciError> {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut s = String::new();
    match io::stdin().read_line(&mut s) {
        Ok(_) => Ok(s.trim().to_string()),
        Err(_) => Err(HciError::ErrorReadingStdin),
    }
}

pub fn prompt_u8(prompt: &str) -> Result<u8, HciError> {
    loop {
        match get_input(prompt) {
            Ok(input) => {
                match input.parse::<u8>() {
                    Ok(num) => return Ok(num),
                    Err(_) => {return Err(HciError::ErrorParsingInput);}
                }
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn prompt_mac(prompt: &str) -> Result<MAC, HciError> {
    loop {
        match get_input(prompt) {
            Ok(input) => {
                match MAC::from_string(&input) {
                    Some(mac) => return Ok(mac),
                    None => return Err(HciError::InvalidDataFormat),
                }
            }
            Err(e) => return Err(e),
        }
    }
}

pub fn prompt_ip(prompt: &str) -> Result<IP, HciError> {
    loop {
        match get_input(prompt) {
            Ok(input) => {
                match IP::from_string(&input) {
                    Some(ip) => return Ok(ip),
                    None => return Err(HciError::InvalidDataFormat),
                }
            }
            Err(e) => return Err(e),
        }
    }
}

pub enum MenuOptions {
    Nothing,
    Exit,
    CreateRouter,
    CreateDevice,
    ShowAll,
    Connection,
    Ping,
}

pub fn menu() -> MenuOptions {
    println!("======= Network Simulator =======");
    println!("1. Create a router");
    println!("2. Create a device");
    println!("3. Show all devices and networks");
    println!("4. Connect two devices");
    println!("5. Ping from a device to an IP");
    println!("0. Quit");
    println!("=================================");
    
    let choice = match prompt_u8("Choose an option: ") {
        Ok(num) => num,
        Err(_) => {
            println!("Error reading input.");
            return MenuOptions::Nothing;
        }
    };

    match choice {
        0 => return MenuOptions::Exit,
        1 => return MenuOptions::CreateRouter,
        2 => return MenuOptions::CreateDevice,
        3 => return MenuOptions::ShowAll,
        4 => return MenuOptions::Connection,
        5 => return MenuOptions::Ping,
        _ => {
            return MenuOptions::Nothing;
        }
    }
}
