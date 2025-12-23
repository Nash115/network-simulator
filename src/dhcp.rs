use crate::ip::IP;
use crate::nic::NIC;


#[derive(Clone)]
pub struct DHCP {
    pub netmask: IP,
    pub first_ip: IP,
    pub last_ip: IP,
}

pub enum DhcpError {
    DisabledDHCP,
    NotEnoughAddressableIPs,
    IPsNotInSameNetwork,
    InvalidIPsRange,
    NoMoreIPsAvailable,
    NoDHCPServerFound
}

impl std::fmt::Display for DhcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DhcpError::DisabledDHCP => write!(f, "Disabled DHCP"),
            DhcpError::NotEnoughAddressableIPs => write!(f, "Not Enough Addressable IPs"),
            DhcpError::IPsNotInSameNetwork => write!(f, "IPs Not In Same Network"),
            DhcpError::InvalidIPsRange => write!(f, "Invalid IPs Range"),
            DhcpError::NoMoreIPsAvailable => write!(f, "No More IPs Available"),
            DhcpError::NoDHCPServerFound => write!(f, "No DHCP Server accessible"),
        }
    }
}

impl DHCP {
    pub fn new(nic: NIC, first_ip:IP, last_ip:IP) -> Result<Self, DhcpError> {
        // Check 1 : It must have at least 2 addressable IPs on the network
        if nic.total_addressable_ips() < 2 {
            return Err(DhcpError::NotEnoughAddressableIPs);
        }
        // Check 2 : The start and end IP addresses must be in the same network as the NIC
        let nic_start: NIC = NIC::new(
            first_ip.clone(),
            nic.netmask.clone(),
        );
        let nic_end: NIC = NIC::new(
            last_ip.clone(),
            nic.netmask.clone(),
        );
        if ! nic.same_network(nic_start) || ! nic.same_network(nic_end) {
            return Err(DhcpError::IPsNotInSameNetwork);
        }
        // Check 3 : The IP range must be valid
        if ! last_ip.is_greater_than(&first_ip) {
            return Err(DhcpError::InvalidIPsRange);
        }
        // Ok
        Ok (DHCP {
            netmask: nic.netmask.clone(),
            last_ip: last_ip.clone(),
            first_ip: first_ip.clone()
        })
    }
}
