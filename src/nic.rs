use crate::colors::Colors;
use crate::ip::IP;
use crate::mac::MAC;


#[derive(Clone)]
pub struct NIC {
    pub ip: IP,
    pub netmask: IP,
    pub mac: MAC
}

pub enum IpAddressType { 
    NetworkAddress,
    BroadcastAddress,
    HostAddress
}

impl std::fmt::Display for NIC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}/{}{} - {}", self.ip, Colors::CYAN, self.netmask_u8(), Colors::RESET, self.mac)
    }
}

impl NIC {
    pub fn new(ip: IP, netmask: IP) -> Self {
        Self {
            ip,
            netmask,
            mac: MAC::new()
        }
    }

    pub fn network_address_bin(&self) -> String {
        let ip_bin: String = self.ip.to_bin_ddn();
        let netmask_bin: String = self.netmask.to_bin_ddn();
        let mut network_address_bin: String = String::new();
        for (i, c) in netmask_bin.chars().enumerate() {
            if c == '1' {
                network_address_bin.push(ip_bin.chars().nth(i).unwrap());
            } else if c == '0' {
                network_address_bin.push('0');
            } else if c == '.' {
                network_address_bin.push('.');
            }
        }
        return network_address_bin;
    }

    pub fn network_address(&self) -> IP {
        let network_address_bin: String = self.network_address_bin();
        let parts: Vec<&str> = network_address_bin.split('.').collect();
        let mut octets: [u8; 4] = [0; 4];
        for i in 0..4 {
            octets[i] = u8::from_str_radix(parts[i], 2).unwrap();
        }
        IP::V4(octets[0], octets[1], octets[2], octets[3])
    }

    pub fn netmask_u8(&self) -> u8 {
        let netmask_bin = self.netmask.to_bin_ddn();
        let mut cpt: u8 = 0;
        for c in netmask_bin.chars() {
            if c == '1' {
                cpt += 1;
            } else if c == '0' {
                break;
            }
        }
        return cpt;
    }

    pub fn total_addressable_ips(&self) -> i32 {
        let base:i32 = 2;
        let netmask: i32 = self.netmask_u8().into();
        if (32 - netmask) <= 1 {
            return 0;
        }
        base.pow((32 - netmask).try_into().unwrap()) - 2
    }

    pub fn same_network(&self, r2: NIC) -> bool {
        return self.network_address_bin() == r2.network_address_bin();
    }

    pub fn ip_address_type(&self) -> IpAddressType {
        let mut next_ip = self.ip.clone();
        if self.ip == self.network_address() {
            return IpAddressType::NetworkAddress;
        }
        match next_ip.increment() {
            Ok(_) => {
                if self.same_network(NIC {ip:next_ip, netmask:self.netmask.clone(), mac:MAC::new()}) {
                    return IpAddressType::HostAddress;
                } else {
                    return IpAddressType::BroadcastAddress;
                }
            }
            Err(_) => {
                return IpAddressType::BroadcastAddress;
            }
        }
    }

    pub fn set_localhost(&mut self) {
        self.ip = IP::V4(127, 0, 0, 1);
        self.netmask = IP::V4(255, 0, 0, 0);
    }

    pub fn is_localhost(&self) -> bool {
        match self.ip {
            IP::V4(a, _, _, _) if a == 127 => true,
            _ => false
        }
    }
}
