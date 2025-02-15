// src/lib.rs

// Re-export the load balancer module and its types
pub mod load_balancer;
pub use load_balancer::least_connections_lb::LeastConnectionsStrategy;
pub use load_balancer::round_robin_lb::RoundRobinStrategy;
pub use load_balancer::weighted_round_robin_lb::WeightedRoundRobin;
pub use load_balancer::factory::{LoadBalancerFactory, LoadBalancer, SelectedLB};

// Re-export the types module (if it exists)
pub mod types;
pub use types::LbAlgorithm;

pub mod proxy_service;