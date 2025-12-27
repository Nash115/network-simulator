use std::fs::{read_to_string};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoadedRouter {
    pub name: String,
    pub lan: LoadedRouterInterface,
    pub wan: LoadedRouterInterface
}

#[derive(Serialize, Deserialize)]
pub struct LoadedRouterInterface {
    pub ip: String,
    pub netmask: u8,
    pub mac: Option<String>,
    pub dhcp: Option<LoadedRouterDHCP>,
}

#[derive(Serialize, Deserialize)]
pub struct LoadedRouterDHCP {
    pub first_ip: String,
    pub last_ip: String
}


#[derive(Serialize, Deserialize)]
pub struct LoadedDevices {
    pub name: String,
    pub ip: Option<String>,
    pub netmask: Option<u8>,
    pub mac: Option<String>
}


#[derive(Serialize, Deserialize)]
pub struct LoadedConnections {
    pub from: String,
    pub to: String
}


#[derive(Serialize, Deserialize)]
pub struct LoadedData {
    pub routers: Option<Vec<LoadedRouter>>,
    pub devices: Option<Vec<LoadedDevices>>,
    pub connections: Option<Vec<LoadedConnections>>
}


pub enum LoadError {
    FileNotFound(String),
    ParseError(String, String)
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::FileNotFound(file) => write!(f, "File {} not found", file),
            LoadError::ParseError(file, error) => write!(f, "Error parsing {} : {}", file, error),
        }
    }
}


fn file_exists(file_path: &str) -> bool {
    std::path::Path::new(file_path).exists()
}

pub fn load_data(file_path: &str) -> Result<LoadedData, LoadError> {
    if !file_exists(file_path) {
        return Err(LoadError::FileNotFound(file_path.to_string()));
    }

    let file_content = match read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return Err(LoadError::FileNotFound(file_path.to_string())),
    };

    let loaded_data: LoadedData = match serde_yaml::from_str(&file_content) {
        Ok(data) => data,
        Err(e) => {
            return Err(LoadError::ParseError(file_path.to_string(), e.to_string()))
        }
    };

    Ok(loaded_data)
}
