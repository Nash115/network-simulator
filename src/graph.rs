use crate::colors::Colors;
use crate::device::Device;
use crate::dhcp::DHCP;
use crate::dhcp::DhcpError;
use crate::ip::IP;
use crate::load::LoadedData;
use crate::mac::MAC;
use crate::nic::NIC;
use crate::router::{Router, RouterInterface};


#[derive(Clone)]
pub enum NodeType {
    Router,
    Device
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Router => write!(f, "Router"),
            NodeType::Device => write!(f, "Device"),
        }
    }
}

pub enum GraphError {
    AlreadyExistingMacAddress,
    ConnectionAlreadyExists,
    MaxConnectionReached(MAC),
    ConnectionNotPossible
}

impl std::fmt::Display for GraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphError::AlreadyExistingMacAddress => write!(f, "Already Existing MAC Address"),
            GraphError::ConnectionAlreadyExists => write!(f, "Connection Already Exists"),
            GraphError::MaxConnectionReached(mac) => write!(f, "Max Connection Reached for MAC {}", mac),
            GraphError::ConnectionNotPossible => write!(f, "Connection Not Possible (Incompatible networks and no DHCP available)")
        }
    }
}

#[derive(Clone)]
pub struct Graph {
    nodes: Vec<(MAC, NodeType)>,
    connections: Vec<(MAC, MAC)>,
    pub routers: Vec<Router>,
    pub devices: Vec<Device>
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            connections: Vec::new(),
            routers: Vec::new(),
            devices: Vec::new()
        }
    }

    pub fn append_router(&mut self, router: Router) -> Result<(), GraphError> {
        if self.node_type_with_mac(router.nic_lan.mac.clone()).is_some() || self.node_type_with_mac(router.nic_wan.mac.clone()).is_some() {
            return Err(GraphError::AlreadyExistingMacAddress);
        }
        self.nodes.push((router.nic_lan.mac.clone(), NodeType::Router));
        self.nodes.push((router.nic_wan.mac.clone(), NodeType::Router));
        self.routers.push(router);
        Ok(())
    }

    pub fn append_device(&mut self, device: Device) -> Result<(), GraphError> {
        if self.node_type_with_mac(device.nic.mac.clone()).is_some() {
            return Err(GraphError::AlreadyExistingMacAddress);
        }
        self.nodes.push((device.nic.mac.clone(), NodeType::Device));
        self.devices.push(device);
        Ok(())
    }

    pub fn append_connection(&mut self, nic1: NIC, nic2: NIC) -> Result<(), GraphError> {
        if self.are_connected(nic1.mac.clone(), nic2.mac.clone()) {
            return Err(GraphError::ConnectionAlreadyExists);
        }
        for mac in &[nic1.mac.clone(), nic2.mac.clone()] {
            let connections = self.connections(mac.clone());
            let node_t = self.node_type_with_mac(mac.clone());
            if let Some(t) = node_t {
                match t {
                    NodeType::Device => {
                        if connections.len() >= 1 {
                            return Err(GraphError::MaxConnectionReached(mac.clone()));
                        }
                    },
                    NodeType::Router => {
                        if connections.len() >= 2 {
                            return Err(GraphError::MaxConnectionReached(mac.clone()));
                        }
                    }
                }
            } else {
                return Err(GraphError::ConnectionNotPossible);
            }
        }
        if !nic1.same_network(nic2.clone()) {
            return Err(GraphError::ConnectionNotPossible);
        }
        self.connections.push((nic1.mac.clone(), nic2.mac.clone()));
        Ok(())
    }

    pub fn append_internal_router_connection(&mut self, mac_lan: MAC, mac_wan: MAC) -> Result<(), GraphError> {
        if self.are_connected(mac_lan.clone(), mac_wan.clone()) {
            return Err(GraphError::ConnectionAlreadyExists);
        }
        self.connections.push((mac_lan.clone(), mac_wan.clone()));
        Ok(())
    }

    pub fn show(&self) {
        println!("--- Network Graph ---");
        println!("Nodes:");
        for (mac, device_type) in &self.nodes {
            println!("- {} ({})", mac, device_type);
        }
        println!("Connections:");
        for (mac1, mac2) in &self.connections {
            println!("- {} <-> {}", mac1, mac2);
        }

        println!("\n--- Routers ---");
        for router in &self.routers {
            println!("{}", router);
        }
        println!("\n--- Devices ---");
        for device in &self.devices {
            println!("{}", device);
        }
    }

    pub fn neighbors(&self, mac: MAC) -> Vec<MAC> {
        let mut neighbors: Vec<MAC> = Vec::new();
        for (mac1, mac2) in &self.connections {
            if *mac1 == mac {
                neighbors.push(mac2.clone());
            } else if *mac2 == mac {
                neighbors.push(mac1.clone());
            }
        }
        neighbors
    }

    pub fn connections(&self, mac: MAC) -> Vec<(MAC, MAC)> {
        let mut connections = Vec::new();
        for (mac1, mac2) in &self.connections {
            if *mac1 == mac || *mac2 == mac {
                connections.push((mac1.clone(), mac2.clone()));
            }
        }
        connections
    }

    pub fn are_connected(&self, mac1: MAC, mac2: MAC) -> bool {
        for (m1, m2) in &self.connections {
            if (*m1 == mac1 && *m2 == mac2) || (*m1 == mac2 && *m2 == mac1) {
                return true;
            }
        }
        false
    }

    pub fn breadth_first_search(&self, start:MAC) -> Vec<MAC> {
        let mut visited: Vec<MAC> = Vec::new();
        let mut queue: Vec<MAC> = Vec::new();
        queue.push(start.clone());
        visited.push(start.clone());

        while !queue.is_empty() {
            let mut queue_temp: Vec<MAC> = Vec::new();
            for current_mac in &queue {
                for neighbor in self.neighbors(current_mac.clone()) {
                    if ! visited.contains(&neighbor) {
                        queue_temp.push(neighbor.clone());
                        visited.push(neighbor.clone());
                    }
                }
            }
            queue = queue_temp;
        }
        visited
    }

    pub fn breadth_first_search_ip(&self, start:MAC, ip: IP) -> bool {
        let accessibles = self.breadth_first_search(start);
        for mac in accessibles {
            let t = self.node_type_with_mac(mac.clone());
            match t {
                Some(NodeType::Device) => {
                    for device in &self.devices {
                        if device.nic.mac == mac && device.nic.ip == ip {
                            return true;
                        }
                    }
                },
                Some(NodeType::Router) => {
                    for router in &self.routers {
                        if router.nic_lan.mac == mac && router.nic_lan.ip == ip {
                            return true;
                        }
                        if router.nic_wan.mac == mac && router.nic_wan.ip == ip {
                            return true;
                        }
                    }
                },
                None => {}
            }
        }
        false
    }

    pub fn breadth_first_search_and_dhcp_connection(&mut self, nic_src:&mut NIC, nic_dest: &NIC) -> Result<IP, DhcpError> {
        let mut last_dhcp_error: DhcpError = DhcpError::NoDHCPServerFound;
        let accessibles = self.breadth_first_search(nic_dest.mac.clone());
        for mac in accessibles {
            let router_idx = self.routers.iter().position(|r| (r.nic_lan.mac == mac) || (r.nic_wan.mac == mac));
            if let Some(idx) = router_idx {
                let interface: RouterInterface = if self.routers[idx].nic_lan.mac == mac {
                    RouterInterface::LAN
                } else {
                    RouterInterface::WAN
                };
                let dhcp = match interface {
                    RouterInterface::LAN => &self.routers[idx].dhcp_lan,
                    RouterInterface::WAN => &self.routers[idx].dhcp_wan,
                };
                if dhcp.is_some() {
                    let netmask = dhcp.as_ref().unwrap().netmask.clone();
                    let tmp_graph = self.clone();
                    let ip_r = self.routers[idx].get_next_dhcp_ip(&tmp_graph, interface);
                    match ip_r {
                        Ok(ip) => {
                            nic_src.ip = ip.clone();
                            nic_src.netmask = netmask;
                            return Ok(ip)
                        },
                        Err(e) => {
                            last_dhcp_error = e;
                        }
                    }
                }
            }
        }
        Err(last_dhcp_error)
    }

    pub fn node_type_with_mac(&self, mac: MAC) -> Option<NodeType> {
        for (node_mac, device_type) in &self.nodes {
            if *node_mac == mac {
                return Some(device_type.clone());
            }
        }
        None
    }

    fn search_router_with_mac(&self, mac: MAC) -> Option<(Router, RouterInterface)> {
        for router in &self.routers {
            if router.nic_lan.mac == mac {
                return Some((router.clone(), RouterInterface::LAN));
            }
            if router.nic_wan.mac == mac {
                return Some((router.clone(), RouterInterface::WAN));
            }
        }
        None
    }

    fn search_device_with_mac(&self, mac: MAC) -> Option<Device> {
        for device in &self.devices {
            if device.nic.mac == mac {
                return Some(device.clone());
            }
        }
        None
    }

    pub fn nic_with_mac(&self, mac: MAC) -> Option<NIC> {
        return match self.node_type_with_mac(mac.clone()) {
            Some(NodeType::Router) => {
                let router = self.search_router_with_mac(mac.clone());
                match router {
                    Some((r,i)) => {Some(
                        match i {
                            RouterInterface::LAN => r.nic_lan.clone(),
                            RouterInterface::WAN => r.nic_wan.clone(),
                        }
                    )},
                    None => None
                }
            },
            Some(NodeType::Device) => {
                let device = self.search_device_with_mac(mac.clone());
                match device {
                    Some(a) => Some(a.nic.clone()),
                    None => None
                }
            },
            None => None
        };
    }

    pub fn update_nic(&mut self, mac: MAC, new_nic: NIC) -> Result<(), GraphError> {
        match self.node_type_with_mac(mac.clone()) {
            Some(NodeType::Router) => {
                for router in &mut self.routers {
                    if router.nic_lan.mac == mac {
                        router.nic_lan = new_nic;
                        return Ok(());
                    }
                    if router.nic_wan.mac == mac {
                        router.nic_wan = new_nic;
                        return Ok(());
                    }
                }
                Err(GraphError::ConnectionNotPossible)
            },
            Some(NodeType::Device) => {
                for device in &mut self.devices {
                    if device.nic.mac == mac {
                        device.nic = new_nic;
                        return Ok(());
                    }
                }
                Err(GraphError::ConnectionNotPossible)
            },
            None => Err(GraphError::ConnectionNotPossible)
        }
    }

    pub fn load_data(&mut self, loaded_data: LoadedData) {
        let mut routers_loaded: u32 = 0;
        let mut devices_loaded: u32 = 0;
        let mut connections_loaded: u32 = 0;
        let loaded_routers = match loaded_data.routers {
            Some(e) => e,
            _ => {Vec::new()}
        };
        let loaded_devices = match loaded_data.devices {
            Some(e) => e,
            _ => {Vec::new()}
        };
        let loaded_connections = match loaded_data.connections {
            Some(e) => e,
            _ => {Vec::new()}
        };

        for r in loaded_routers {
            let dhcp_lan_first_ip: Option<IP> = match &r.lan.dhcp {
                Some(dhcp) => match IP::from_string(dhcp.first_ip.as_str()) {
                    Some(ip) => Some(ip), None => {
                        println!("{}Skipping{} DHCP for router {} LAN due to invalid first IP.", Colors::YELLOW, Colors::RESET, r.name);
                        None
                    }
                }, _ => None,
            };
            let dhcp_lan_last_ip: Option<IP> = match &r.lan.dhcp {
                Some(dhcp) => match IP::from_string(dhcp.last_ip.as_str()) {
                    Some(ip) => Some(ip), None => {
                        println!("{}Skipping{} DHCP for router {} LAN due to invalid last IP.", Colors::YELLOW, Colors::RESET, r.name);
                        None
                    }
                }, _ => None,
            };
            let dhcp_wan_first_ip: Option<IP> = match &r.wan.dhcp {
                Some(dhcp) => match IP::from_string(dhcp.first_ip.as_str()) {
                    Some(ip) => Some(ip), None => {
                        println!("{}Skipping{} DHCP for router {} WAN due to invalid first IP.", Colors::YELLOW, Colors::RESET, r.name);
                        None
                    }
                }, _ => None,
            };
            let dhcp_wan_last_ip: Option<IP> = match &r.wan.dhcp {
                Some(dhcp) => match IP::from_string(dhcp.last_ip.as_str()) {
                    Some(ip) => Some(ip), None => {
                        println!("{}Skipping{} DHCP for router {} WAN due to invalid last IP.", Colors::YELLOW, Colors::RESET, r.name);
                        None
                    }
                }, _ => None,
            };
            let nic_lan = NIC {
                mac: match r.lan.mac {
                    Some(mac_str) => match MAC::from_string(mac_str.as_str()) {
                        Some(mac) => mac, None => {
                            println!("{}Skipping{} router {} due to invalid LAN MAC.", Colors::YELLOW, Colors::RESET, r.name);
                            continue;
                        }
                    }, None => {
                        MAC::new()
                    }
                },
                ip: match IP::from_string(r.lan.ip.as_str()) {
                    Some(ip) => ip, None => {
                        println!("{}Skipping{} router {} due to invalid LAN IP.", Colors::YELLOW, Colors::RESET, r.name);
                        continue;
                    },
                },
                netmask: IP::from_cidr(r.lan.netmask),
            };
            let nic_wan = NIC {
                mac: match r.wan.mac {
                    Some(mac_str) => match MAC::from_string(mac_str.as_str()) {
                        Some(mac) => mac, None => {
                            println!("{}Skipping{} router {} due to invalid WAN MAC.", Colors::YELLOW, Colors::RESET, r.name);
                            continue;
                        }
                    }, None => {
                        MAC::new()
                    }
                },
                ip: match IP::from_string(r.wan.ip.as_str()) {
                    Some(ip) => ip, None => {
                        println!("{}Skipping{} router {} due to invalid WAN IP.", Colors::YELLOW, Colors::RESET, r.name);
                        continue;
                    },
                },
                netmask: IP::from_cidr(r.wan.netmask),
            };
            match self.append_router( Router {
                name: r.name.clone(),
                nic_lan: nic_lan.clone(),
                nic_wan: nic_wan.clone(),
                dhcp_lan: match (dhcp_lan_first_ip, dhcp_lan_last_ip) {
                    (Some(first_ip), Some(last_ip)) => match DHCP::new(
                        nic_lan,
                        first_ip,
                        last_ip
                    ) { Ok(dhcp) => Some(dhcp),
                        Err(e) => {
                            println!("{}Skipping{} DHCP for router {} LAN due to error: {}", Colors::YELLOW, Colors::RESET, r.name, e);
                            None
                        }
                    }, _ => None,
                },
                dhcp_wan: match (dhcp_wan_first_ip, dhcp_wan_last_ip) {
                    (Some(first_ip), Some(last_ip)) => match DHCP::new(
                        nic_wan,
                        first_ip,
                        last_ip
                    ) { Ok(dhcp) => Some(dhcp),
                        Err(e) => {
                            println!("{}Skipping{} DHCP for router {} WAN due to error: {}", Colors::YELLOW, Colors::RESET, r.name, e);
                            None
                        }
                    }, _ => None,
                }
            }) {
                Ok(_) => {routers_loaded += 1;},
                Err(e) => {
                    println!("{}Error{} adding router {}: {}", Colors::RED, Colors::RESET, r.name, e);
                }
            }
        }

        for d in loaded_devices {
            match self.append_device( Device {
                name: d.name.clone(),
                nic: NIC {
                    mac: match d.mac {
                        Some(mac_str) => match MAC::from_string(mac_str.as_str()) {
                            Some(mac) => mac, None => {
                                println!("{}Skipping{} device {} due to invalid MAC.", Colors::YELLOW, Colors::RESET, d.name);
                                continue;
                            }
                        }, None => {
                            MAC::new()
                        }
                    },
                    ip: match d.ip {
                        Some(ip_str) => match IP::from_string(ip_str.as_str()) {
                            Some(ip) => ip, None => {
                                println!("{}Skipping{} device {} due to invalid IP.", Colors::YELLOW, Colors::RESET, d.name);
                                continue;
                            }
                        }, None => {
                            IP::V4(127, 0, 0, 1)
                        }
                    },
                    netmask: match d.netmask {
                        Some(cidr) => IP::from_cidr(cidr),
                        None => IP::from_cidr(8),
                    },
                }
            } ) {
                Ok(_) => {devices_loaded += 1;},
                Err(e) => {
                    println!("{}Error{} adding device {}: {}", Colors::RED, Colors::RESET, d.name, e);
                }
            }
        }

        for c in loaded_connections {
            let mac_src = match MAC::from_string(&c.from) {
                Some(mac) => mac, None => {
                    println!("{}Skipping{} connection from {} to {} due to invalid source MAC.", Colors::YELLOW, Colors::RESET, c.from, c.to);
                    continue;
                }
            };
            let mac_dest = match MAC::from_string(&c.to) {
                Some(mac) => mac, None => {
                    println!("{}Skipping{} connection from {} to {} due to invalid destination MAC.", Colors::YELLOW, Colors::RESET, c.from, c.to);
                    continue;
                }
            };
            match connection_with_mac(self,
                mac_src.clone(),
                mac_dest.clone()
            ) {
                true => { connections_loaded += 1;},
                false => {
                    println!("Trying to reconnect with reversed order of MACs...");
                    match connection_with_mac(self, mac_dest.clone(), mac_src.clone()) {
                        true => { connections_loaded += 1; },
                        false => println!("{}Error{} connecting {} to {} (see above)", Colors::RED, Colors::RESET, c.from, c.to)
                    }
                }
            }
        }
        println!("{}Loaded{} {} routers, {} devices, and {} connections.", Colors::BLUE, Colors::RESET, routers_loaded, devices_loaded, connections_loaded);
    }

}

pub fn connection_with_mac(graph: &mut Graph, mac_src: MAC, mac_dest: MAC) -> bool {
    let mut nic_src = match graph.nic_with_mac(mac_src.clone()) {
        Some(nic) => nic,
        None => {
            println!("Device with MAC address '{}' not found.", mac_src);
            return false;
        }
    };
    let nic_src_original = nic_src.clone();
    let nic_dest = match graph.nic_with_mac(mac_dest.clone()) {
        Some(nic) => nic,
        None => {
            println!("Device with MAC address '{}' not found.", mac_dest);
            return false;
        }
    };
    if !nic_src.same_network(nic_dest.clone()) {
        if graph.connections(nic_src.mac.clone()).is_empty() {
            nic_src.set_localhost();
        }
        if !nic_src.is_localhost() {
            println!("Devices are not on the same network and {} could not connect to the network.", nic_src.mac);
            return false;
        }
        match graph.breadth_first_search_and_dhcp_connection(&mut nic_src, &nic_dest) {
            Ok(ip) => {
                println!("DHCP attribution succeed : {}", ip);
                if let Err(e) = graph.update_nic(mac_src.clone(), nic_src.clone()) {
                    println!("Error updating NIC: {}", e);
                    return false;
                }
            }
            Err(e) => {
                println!("Error during DHCP attribution : {}", e);
                return false;
            }
        }
    }
    match graph.append_connection(nic_src, nic_dest) {
        Ok(_) => return true,
        Err(e) => {
            println!("Error connecting devices: {}", e);
            match graph.update_nic(mac_src.clone(), nic_src_original.clone()) {
                Ok(_) => {},
                Err(e) => println!("Additionally, error reverting NIC changes: {}", e),
            }
        }
    }
    return false;
}
