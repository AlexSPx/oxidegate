pub mod load_balancer;
pub use load_balancer::least_connections_lb::LeastConnectionsStrategy;
pub use load_balancer::round_robin_lb::RoundRobinStrategy;
pub use load_balancer::weighted_round_robin_lb::WeightedRoundRobin;
pub use load_balancer::factory::{LoadBalancerFactory, LoadBalancer, SelectedLB};

pub mod types;
pub use types::LbAlgorithm;

pub mod proxy_service;