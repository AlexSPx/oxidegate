use std::sync::Arc;
use crate::types::LbAlgorithm;

use super::round_robin_lb::RoundRobinStrategy;

#[async_trait::async_trait]
pub trait LoadBalancer: Send + Sync {
    async fn next(&self) -> Option<String>;
}

pub struct LoadBalancerFactory;

impl LoadBalancerFactory {
    pub fn create(algorithm: LbAlgorithm, servers: Vec<String>) -> Arc<dyn LoadBalancer> {
        match algorithm {
            LbAlgorithm::RoundRobin => Arc::new(RoundRobinStrategy::new(servers)),
            LbAlgorithm::LeastConnections => unimplemented!(),
        }
    }
}