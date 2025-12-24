mod colors;
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
use crate::hci::{menu, MenuOptions};
use crate::ip::IP;
use crate::router::create_router;


fn main() {
    let mut graph: Graph = Graph::new();
    let mut running = true;

    while running {
        let mut wait = true;
        hci::clear_screen();
        match menu() {
            MenuOptions::Exit => {
                wait = false;
                println!("Goodbye!");
                running = false;
            },
            MenuOptions::CreateRouter => {
                match hci::create_router_interactive(&mut graph) {
                    true => println!("Router created successfully."),
                    false => {},
                }
            },
            MenuOptions::CreateDevice => {
                match hci::create_device_interactive(&mut graph) {
                    true => println!("Device created successfully."),
                    false => {},
                }
            },
            MenuOptions::ShowAll => {
                graph.show();
            },
            MenuOptions::Connection => {
                match hci::connection_interactive(&mut graph) {
                    true => println!("Devices connected successfully."),
                    false => {},
                }
            },
            MenuOptions::Ping => {
                hci::ping_interactive(&graph);
            }
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
