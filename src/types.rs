use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Frontend {
    #[serde(rename = "path_prefixes")]
    pub path_prefix: Vec<String>,
    pub backend: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Backend {
    pub name: String,
    pub servers: Vec<String>,
    #[serde(default = "default_lb_algorithm")]
    pub lb_algorithm: LbAlgorithm,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
pub enum LbAlgorithm {
    RoundRobin,
    LeastConnections,
}

fn default_lb_algorithm() -> LbAlgorithm { LbAlgorithm::RoundRobin }