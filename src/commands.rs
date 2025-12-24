use crate::graph::Graph;
use crate::ip::IP;
use crate::nic::NIC;


pub enum PingStatus {
    Success,
    Timeout,
}

impl std::fmt::Display for PingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PingStatus::Success => write!(f, "Success"),
            PingStatus::Timeout => write!(f, "Timeout")
        }
    }
}

pub fn ping(graph: &Graph, src: NIC, dest: IP) -> PingStatus {
    if !graph.breadth_first_search_ip(src.mac.clone(), dest.clone()) {
        return PingStatus::Timeout;
    }
    PingStatus::Success
}
