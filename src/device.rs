use crate::graph::{Graph, GraphError};
use crate::ip::IP;
use crate::nic::NIC;


#[derive(Clone)]
pub struct Device {
    pub name: String,
    pub nic: NIC
}

impl Device {
    fn new(name: String) -> Self {
        Self {
            name: name.to_string(),
            nic: NIC::new(
                IP::V4(127, 0, 0, 1),
                IP::V4(255, 0, 0, 0),
            )
        }
    }

    pub fn status(&self) {
        println!("DEVICE {} - {} - {}", self.name, self.nic.cidr(), self.nic.mac.to_hex());
    }
}

pub fn create_device(name: String, graph: &mut Graph) -> Result<(), GraphError> {
    let device = Device::new(name);
    graph.append_device(device)
}
