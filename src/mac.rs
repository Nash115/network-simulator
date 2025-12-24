use rand::Rng;

use crate::colors::Colors;

#[derive(Clone)]
pub enum MAC {
    EUI48(u8, u8, u8, u8, u8, u8)
}

impl std::fmt::Display for MAC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", Colors::BRIGHT_MAGENTA, self.to_hex(), Colors::RESET)
    }
}

impl MAC {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        MAC::EUI48(
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
        )
    }

    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return None;
        }
        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            match u8::from_str_radix(part, 16) {
                Ok(byte) => bytes[i] = byte,
                Err(_) => return None,
            }
        }
        Some(MAC::EUI48(bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]))
    }
    
    pub fn to_hex(&self) -> String {
        match self {
            MAC::EUI48(a, b, c, d, e, f) => format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}", a, b, c, d, e, f),
        }
    }
}

impl PartialEq for MAC {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MAC::EUI48(a1, b1, c1, d1, e1, f1), MAC::EUI48(a2, b2, c2, d2, e2, f2)) => {
                a1 == a2 && b1 == b2 && c1 == c2 && d1 == d2 && e1 == e2 && f1 == f2
            }
        }
    }
}
