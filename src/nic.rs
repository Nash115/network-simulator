use crate::ip::IP;
use crate::mac::MAC;


#[derive(Clone)]
pub struct NIC {
    pub ip: IP,
    pub netmask: IP,
    pub mac: MAC
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

    pub fn cidr(&self) -> String {
        format!("{}/{}", self.ip.to_ddn(), self.netmask_u8())
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
