use std::io::{self, Write};

use crate::commands::ping;
use crate::device::create_device;
use crate::graph::{Graph, connection_with_mac};
use crate::ip::IP;
use crate::load::load_data;
use crate::mac::MAC;
use crate::router::{create_router, RouterInterface};


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

pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
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

pub fn prompt_confirmation(prompt: &str) -> Result<bool, HciError> {
    loop {
        match get_input(format!("{} (y/n): ", prompt).as_str()) {
            Ok(input) => {
                match input.to_lowercase().as_str() {
                    "y" | "yes" => return Ok(true),
                    "n" | "no" => return Ok(false),
                    _ => {
                        println!("Please enter 'y' or 'n'.");
                    }
                }
            }
            Err(e) => return Err(e),
        }
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
    Load
}

pub fn menu() -> MenuOptions {
    println!("======= Network Simulator =======");
    println!("1. Create a router");
    println!("2. Create a device");
    println!("3. Show all devices and networks");
    println!("4. Connect two devices");
    println!("5. Ping from a device to an IP");
    println!("6. Load data from yaml file");
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
        6 => return MenuOptions::Load,
        _ => {
            return MenuOptions::Nothing;
        }
    }
}


pub fn create_nic_router_interactive(interface: RouterInterface) -> Option<(IP, IP)> {
    let ip = match prompt_ip(&format!("Enter IP address for interface {}: ", interface)) {
        Ok(ip) => ip,
        Err(e) => {
            println!("Error reading IP address: {}", e);
            return None;
        }
    };
    let netmask_cidr = match prompt_u8(&format!("Enter netmask (CIDR notation) for interface {}: ", interface)) {
        Ok(n) => n,
        Err(e) => {
            println!("Error reading netmask: {}", e);
            return None;
        }
    };
    let netmask = IP::from_cidr(netmask_cidr);
    Some((ip, netmask))
}

pub fn create_dhcp_router_interactive(interface: RouterInterface) -> Option<(Option<IP>, Option<IP>)> {
    let dhcp: bool = match prompt_confirmation(&format!("Enable DHCP on interface {}?", interface)) {
        Ok(ans) => ans,
        Err(e) => {
            println!("Error reading input: {}", e);
            return None;
        }
    };
    if dhcp {
        let first_ip = match prompt_ip(&format!("Enter first DHCP IP for interface {}: ", interface)) {
            Ok(ip) => ip,
            Err(e) => {
                println!("Error reading IP address: {}", e);
                return None;
            }
        };
        let last_ip = match prompt_ip(&format!("Enter last DHCP IP for interface {}: ", interface)) {
            Ok(ip) => ip,
            Err(e) => {
                println!("Error reading IP address: {}", e);
                return None;
            }
        };
        Some((Some(first_ip), Some(last_ip)))
    } else {
        Some((None, None))
    }
}

pub fn create_router_interactive(graph: &mut Graph) -> bool {
    let name: String = match get_input("Enter the router name: ") {
        Ok(n) => n,
        Err(e) => {
            let default_name = "Unnamed Router".to_string();
            println!("Error reading name ({}). Using default name '{}'.", e, default_name);
            default_name
        }
    };

    // NIC LAN
    let (ip_lan, netmask_lan) = match create_nic_router_interactive(RouterInterface::LAN) {
        Some((ip, netmask)) => (ip, netmask),
        None => { return false; }
    };

    // NIC WAN
    let (ip_wan, netmask_wan) = match create_nic_router_interactive(RouterInterface::WAN) {
        Some((ip, netmask)) => (ip, netmask),
        None => { return false; }
    };

    // DHCP LAN
    let (dhcp_lan_first_ip, dhcp_lan_last_ip) = match create_dhcp_router_interactive(RouterInterface::LAN) {
        Some((first_ip, last_ip)) => (first_ip, last_ip),
        None => { return false; }
    };

    // DHCP WAN
    let (dhcp_wan_first_ip, dhcp_wan_last_ip) = match create_dhcp_router_interactive(RouterInterface::WAN) {
        Some((first_ip, last_ip)) => (first_ip, last_ip),
        None => { return false; }
    };

    // Create the router
    match create_router(
        name, graph,
        ip_lan, netmask_lan, ip_wan, netmask_wan,
        dhcp_lan_first_ip, dhcp_lan_last_ip,
        dhcp_wan_first_ip, dhcp_wan_last_ip
    ) {
        Ok(_) => return true,
        Err(e) => println!("Error creating router: {}", e),
    }
    return false;
}

pub fn create_device_interactive(graph: &mut Graph) -> bool {
    let name: String = match get_input("Enter the device name: ") {
        Ok(n) => n,
        Err(e) => {
            let default_name = "Unnamed Device".to_string();
            println!("Error reading name ({}). Using default name '{}'.", e, default_name);
            default_name
        }
    };
    match create_device(name, graph) {
        Ok(_) => return true,
        Err(e) => println!("Error creating device: {}", e),
    }
    return false;
}

pub fn connection_interactive(graph: &mut Graph) -> bool {
    let mac_src = match prompt_mac("Enter the MAC address of the device to connect: ") {
        Ok(mac) => mac,
        Err(e) => {
            println!("Error reading MAC address: {}", e);
            return false;
        }
    };
    let mac_dest = match prompt_mac("Enter the MAC address of the device to connect to: ") {
        Ok(mac) => mac,
        Err(e) => {
            println!("Error reading MAC address: {}", e);
            return false;
        }
    };
    connection_with_mac(graph, mac_src, mac_dest)
}

pub fn ping_interactive(graph: &Graph) -> bool {
    let source_mac = match prompt_mac("Enter the MAC address of the source device: ") {
        Ok(mac) => mac,
        Err(e) => {
            println!("Error reading MAC address: {}", e);
            return false;
        }
    };
    let destination_ip = match prompt_ip("Enter the destination IP address: ") {
        Ok(ip) => ip,
        Err(e) => {
            println!("Error reading IP address: {}", e);
            return false;
        }
    };
    match graph.nic_with_mac(source_mac.clone()) {
        Some(nic) => {
            let status = ping(&graph, nic, destination_ip);
            println!("Ping status: {}", status);
            return true;
        },
        None => {
            println!("Device with MAC address '{}' not found.", source_mac);
            return false;
        }
    };
}

pub fn load_interactive(graph: &mut Graph) -> bool {
    let file_path = match get_input("Enter the path to the YAML file to load : ") {
        Ok(name) => name,
        Err(e) => {
            println!("Error reading file path: {}", e);
            return false;
        }
    };
    let data = load_data(&file_path);
    match data {
        Ok(data) => graph.load_data(data),
        Err(e) => {
            println!("Error loading data: {}", e);
            return false;
        }
    };
    true
}
