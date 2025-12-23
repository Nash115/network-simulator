use crate::graph::Graph;
use crate::ip::IP;
use crate::nic::NIC;


pub enum PingStatus {
    Success,
    HostUnreachable,
    Timeout,
}

impl std::fmt::Display for PingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PingStatus::Success => write!(f, "Success"),
            PingStatus::HostUnreachable => write!(f, "Host Unreachable"),
            PingStatus::Timeout => write!(f, "Timeout")
        }
    }
}

pub fn ping(graph: &Graph, src: NIC, dest: IP) -> PingStatus {
    let src_nic = NIC::new(src.ip.clone(), src.netmask.clone());
    let dest_nic = NIC::new(dest.clone(), src.netmask.clone());
    if !src_nic.same_network(dest_nic) {
        return PingStatus::HostUnreachable;
    }
    if !graph.breadth_first_search_ip(src.mac.clone(), dest.clone()) {
        return PingStatus::Timeout;
    }
    PingStatus::Success
}