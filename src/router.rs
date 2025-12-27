use crate::commands::{PingStatus, ping};
use crate::dhcp::{DHCP, DhcpError};
use crate::graph::{Graph, GraphError};
use crate::ip::IP;
use crate::nic::NIC;


#[derive(Clone)]
pub struct Router {
    pub name: String,
    pub nic_lan: NIC,
    pub nic_wan: NIC,
    pub dhcp_lan: Option<DHCP>,
    pub dhcp_wan: Option<DHCP>,
}

impl std::fmt::Display for Router {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ROUTER {} - LAN:[{}] WAN:[{}]\n\tDHCP LAN -> {}\n\tDHCP WAN -> {}",
            self.name, self.nic_lan, self.nic_wan,
            match &self.dhcp_lan {
                Some(dhcp) => format!(" Enabled : {} > {}", dhcp.first_ip, dhcp.last_ip),
                None => "Disabled".to_string(),
            },
            match &self.dhcp_wan {
                Some(dhcp) => format!(" Enabled : {} > {}", dhcp.first_ip, dhcp.last_ip),
                None => "Disabled".to_string(),
            }
        )
    }
}

pub enum RouterInterface {LAN,WAN}

impl std::fmt::Display for RouterInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterInterface::LAN => write!(f, "LAN"),
            RouterInterface::WAN => write!(f, "WAN"),
        }
    }
}

impl Router {
    fn new( name: String,
            ip_lan:IP, netmask_lan:IP, ip_wan:IP, netmask_wan:IP,
            dhcp_lan_first_ip: Option<IP>, dhcp_lan_last_ip: Option<IP>,
            dhcp_wan_first_ip: Option<IP>, dhcp_wan_last_ip: Option<IP>) -> Self {
        let nic_lan: NIC = NIC::new(ip_lan, netmask_lan);
        let nic_wan: NIC = NIC::new(ip_wan, netmask_wan);
        let dhcp_lan: Option<DHCP> = match (dhcp_lan_first_ip, dhcp_lan_last_ip) {
            (Some(first_ip), Some(last_ip)) => {
                match DHCP::new(nic_lan.clone(), first_ip, last_ip) {
                    Ok(dhcp) => Some(dhcp),
                    Err(_) => None,
                }
            },
            _ => None,
        };
        let dhcp_wan: Option<DHCP> = match (dhcp_wan_first_ip, dhcp_wan_last_ip) {
            (Some(first_ip), Some(last_ip)) => {
                match DHCP::new(nic_wan.clone(), first_ip, last_ip) {
                    Ok(dhcp) => Some(dhcp),
                    Err(_) => None,
                }
            },
            _ => None,
        };
        Self { name, nic_lan, nic_wan, dhcp_lan, dhcp_wan }
    }

    pub fn get_next_dhcp_ip(&mut self, graph: &Graph, interface: RouterInterface) -> Result<IP, DhcpError> {
        let dhcp = match interface {
            RouterInterface::LAN => &mut self.dhcp_lan,
            RouterInterface::WAN => &mut self.dhcp_wan,
        };
        let nic = match interface {
            RouterInterface::LAN => &self.nic_lan,
            RouterInterface::WAN => &self.nic_wan,
        };
        let dhcp = match dhcp {
            Some(dhcp) => dhcp,
            None => return Err(DhcpError::DisabledDHCP)
        };
        let mut ip_candidate: IP = dhcp.first_ip.clone();
        loop {
            if ip_candidate.is_greater_than(&dhcp.last_ip) {
                return Err(DhcpError::NoMoreIPsAvailable);
            }
            match ping(graph, nic.clone(), ip_candidate.clone()) {
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

pub fn create_router( name: String, graph: &mut Graph,
                      ip_lan: IP, netmask_lan: IP, ip_wan: IP, netmask_wan: IP,
                      dhcp_lan_first_ip: Option<IP>, dhcp_lan_last_ip: Option<IP>,
                      dhcp_wan_first_ip: Option<IP>, dhcp_wan_last_ip: Option<IP>) -> Result<(), GraphError> {
    let router = Router::new(
        name, ip_lan, netmask_lan, ip_wan, netmask_wan,
        dhcp_lan_first_ip, dhcp_lan_last_ip, dhcp_wan_first_ip, dhcp_wan_last_ip
    );
    match graph.append_internal_router_connection(router.nic_lan.mac.clone(), router.nic_wan.mac.clone()) {
        Err(e) => return Err(e),
        _ => {}
    }
    graph.append_router(router)
}
