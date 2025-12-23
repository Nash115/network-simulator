use crate::commands::{PingStatus, ping};
use crate::dhcp::{DHCP, DhcpError};
use crate::graph::{Graph, GraphError};
use crate::ip::IP;
use crate::nic::NIC;


#[derive(Clone)]
pub struct Router {
    name: String,
    pub nic: NIC,
    pub dhcp: Option<DHCP>,
}

impl Router {
    fn new(name: String) -> Self {
        let nic: NIC = NIC::new(
            IP::V4(192, 168, 1, 1),
            IP::V4(255, 255, 255, 0)
        );
        let dhcp_result = DHCP::new(
            nic.clone(),
            IP::V4(192, 168, 1, 10),
            IP::V4(192, 168, 1, 100)
        );
        match dhcp_result {
            Ok(dhcp) => {
                Self {
                    name: name.clone(),
                    nic: nic.clone(),
                    dhcp: Some(dhcp)
                }
            },
            Err(e) => {
                println!("Error with DHCP server setup : {}", e);
                Self {
                    name: name.clone(),
                    nic: nic.clone(),
                    dhcp: None
                }
            }
        }
    }

    pub fn status(&self) {
        println!("ROUTER {} - {} - {}", self.name, self.nic.cidr(), self.nic.mac.to_hex());
        print!("\tDHCP -> ");
        match &self.dhcp {
            Some(dhcp) => {
                println!(" Enabled : {} > {}", dhcp.first_ip.to_ddn(), dhcp.last_ip.to_ddn());
            },
            None => {
                println!("Disabled");
            }
        }
    }

    pub fn get_next_dhcp_ip(&mut self, graph: &Graph) -> Result<IP, DhcpError> {
        let dhcp = match &mut self.dhcp {
            Some(dhcp) => dhcp,
            None => return Err(DhcpError::DisabledDHCP)
        };
        let mut ip_candidate: IP = dhcp.first_ip.clone();
        loop {
            if ip_candidate.is_greater_than(&dhcp.last_ip) {
                return Err(DhcpError::NoMoreIPsAvailable);
            }
            match ping(graph, self.nic.clone(), ip_candidate.clone()) {
                PingStatus::Success => {}
                _ => {
                    return Ok(ip_candidate.clone());
                }
            }
            match ip_candidate.increment() {
                Ok(()) => {},
                Err(_) => return Err(DhcpError::NoMoreIPsAvailable)
            };
        }
    }
}

pub fn create_router(name: String, graph: &mut Graph) -> Result<(), GraphError> {
    let router = Router::new(name);
    graph.append_router(router)
}
