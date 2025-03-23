use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerSettings {
    #[serde(default)]
    pub enable_https: bool,
    #[serde(default = "default_port")]
    pub port: u16,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Frontend {
    #[serde(rename = "path_prefixes")]
    pub path_prefix: Vec<String>,
    pub backend: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendServer {
    pub server: String,
    pub weight: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Backend {
    pub name: String,
    pub servers: Vec<BackendServer>,
    #[serde(default = "default_lb_algorithm")]
    pub lb_algorithm: LbAlgorithm,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum LbAlgorithm {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
}

fn default_lb_algorithm() -> LbAlgorithm {
    LbAlgorithm::RoundRobin
}
fn default_port() -> u16 {
    3000
}

impl Default for ServerSettings {
    fn default() -> Self {
        ServerSettings {
            enable_https: false,
            port: default_port(),
            cert_path: None,
            key_path: None,
        }
    }
}
