mod commands;
mod device;
mod dhcp;
mod graph;
mod hci;
mod ip;
mod mac;
mod nic;
mod router;

use crate::device::create_device;
use crate::graph::Graph;
use crate::hci::MenuOptions;
use crate::router::create_router;


fn main() {
    let mut graph: Graph = Graph::new();
    let mut running = true;

    while running {
        let mut wait = true;
        match hci::menu() {
            hci::MenuOptions::Exit => {
                wait = false;
                println!("Goodbye!");
                running = false;
            },
            hci::MenuOptions::CreateRouter => {
                let name: String = match hci::get_input("Enter the router name: ") {
                    Ok(n) => n,
                    Err(e) => {
                        let default_name = "Unnamed Router".to_string();
                        println!("Error reading name ({}). Using default name '{}'.", e, default_name);
                        default_name
                    }
                };
                match create_router(name, &mut graph) {
                    Ok(_) => println!("Router created successfully."),
                    Err(e) => println!("Error creating router: {}", e),
                }
            },
            hci::MenuOptions::CreateDevice => {
                let name: String = match hci::get_input("Enter the device name: ") {
                    Ok(n) => n,
                    Err(e) => {
                        let default_name = "Unnamed Device".to_string();
                        println!("Error reading name ({}). Using default name '{}'.", e, default_name);
                        default_name
                    }
                };
                match create_device(name, &mut graph) {
                    Ok(_) => println!("Device created successfully."),
                    Err(e) => println!("Error creating device: {}", e),
                }
            },
            hci::MenuOptions::ShowAll => {
                graph.show();
            },
            hci::MenuOptions::Connection => {
                let mac_src = match hci::prompt_mac("Enter the MAC address of the device to connect: ") {
                    Ok(mac) => mac,
                    Err(e) => {
                        println!("Error reading MAC address: {}", e);
                        continue;
                    }
                };
                let mac_dest = match hci::prompt_mac("Enter the MAC address of the device to connect to: ") {
                    Ok(mac) => mac,
                    Err(e) => {
                        println!("Error reading MAC address: {}", e);
                        continue;
                    }
                };
                let mut nic_src = match graph.nic_with_mac(mac_src.clone()) {
                    Some(nic) => nic,
                    None => {
                        println!("Device with MAC address '{}' not found.", mac_src.to_hex());
                        continue;
                    }
                };
                let nic_src_original = nic_src.clone();
                let nic_dest = match graph.nic_with_mac(mac_dest.clone()) {
                    Some(nic) => nic,
                    None => {
                        println!("Device with MAC address '{}' not found.", mac_dest.to_hex());
                        continue;
                    }
                };
                if !nic_src.same_network(nic_dest.clone()) {
                    if graph.connections(nic_src.mac.clone()).is_empty() {
                        nic_src.set_localhost();
                    }
                    if !nic_src.is_localhost() {
                        println!("Devices are not on the same network and {} could not connect to the network.", nic_src.mac.to_hex());
                        continue;
                    }
                    match graph.breadth_first_search_and_dhcp_connection(&mut nic_src, &nic_dest) {
                        Ok(ip) => {
                            println!("DHCP attribution succeed : {}", ip.to_ddn());
                            if let Err(e) = graph.update_nic(mac_src.clone(), nic_src.clone()) {
                                println!("Error updating NIC: {}", e);
                                continue;
                            }
                        }
                        Err(e) => {
                            println!("Error during DHCP attribution : {}", e);
                            continue;
                        }
                    }
                }
                match graph.append_connection(nic_src, nic_dest) {
                    Ok(_) => println!("Devices connected successfully."),
                    Err(e) => {
                        println!("Error connecting devices: {}", e);
                        match graph.update_nic(mac_src.clone(), nic_src_original.clone()) {
                            Ok(_) => {},
                            Err(e) => println!("Additionally, error reverting NIC changes: {}", e),
                        }
                    }
                }
            },
            MenuOptions::Ping => {
                let source_mac = match hci::prompt_mac("Enter the MAC address of the source device: ") {
                    Ok(mac) => mac,
                    Err(e) => {
                        println!("Error reading MAC address: {}", e);
                        continue;
                    }
                };
                let destination_ip = match hci::prompt_ip("Enter the destination IP address: ") {
                    Ok(ip) => ip,
                    Err(e) => {
                        println!("Error reading IP address: {}", e);
                        continue;
                    }
                };
                match graph.nic_with_mac(source_mac.clone()) {
                    Some(nic) => {
                        let status = commands::ping(&graph, nic, destination_ip);
                        println!("Ping status: {}", status);
                    },
                    None => {
                        println!("Device with MAC address '{}' not found.", source_mac.to_hex());
                        continue;
                    }
                };
            },
            _ => {
                wait = false;
                println!("Invalid option. Please try again.");
            }
        }
        if wait {
            let _ = hci::get_input("\nPress Enter to continue...");
        }
    }
}
