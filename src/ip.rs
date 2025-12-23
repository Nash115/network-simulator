#[derive(Clone)]
pub enum IP {
    V4(u8, u8, u8, u8)
}

pub enum IpError {
    MaxIPReached
}

impl IP {
    pub fn from_string(s: &str) -> Option<IP> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return None;
        }
        let mut octets: [u8; 4] = [0; 4];
        for i in 0..4 {
            match parts[i].parse::<u8>() {
                Ok(num) => octets[i] = num,
                Err(_) => return None,
            }
        }
        Some(IP::V4(octets[0], octets[1], octets[2], octets[3]))
    }

    pub fn to_ddn(&self) -> String {
        match self {
            IP::V4(a, b, c, d) => format!("{}.{}.{}.{}", a, b, c, d),
        }
    }

    pub fn to_bin_ddn(&self) -> String {
        match self {
            IP::V4(a, b, c, d) => format!("{:08b}.{:08b}.{:08b}.{:08b}", a, b, c, d),
        }
    }

    pub fn is_greater_than(&self, other: &IP) -> bool {
        match (self, other) {
            (IP::V4(a1, b1, c1, d1), IP::V4(a2, b2, c2, d2)) => {
                if a1 != a2 {
                    return a1 > a2;
                } else if b1 != b2 {
                    return b1 > b2;
                } else if c1 != c2 {
                    return c1 > c2;
                } else {
                    return d1 > d2;
                }
            }
        }
    }

    pub fn increment(&mut self) -> Result<(), IpError> {
        match self {
            IP::V4(a, b, c, d) => {
                if *d < 255 {
                    *d += 1;
                } else {
                    *d = 0;
                    if *c < 255 {
                        *c += 1;
                    } else {
                        *c = 0;
                        if *b < 255 {
                            *b += 1;
                        } else {
                            *b = 0;
                            if *a < 255 {
                                *a += 1;
                            } else {
                                return Err(IpError::MaxIPReached);
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

impl PartialEq for IP {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IP::V4(a1, b1, c1, d1), IP::V4(a2, b2, c2, d2)) => {
                a1 == a2 && b1 == b2 && c1 == c2 && d1 == d2
            }
        }
    }
}
