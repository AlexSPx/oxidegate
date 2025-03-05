use std::sync::Arc;
use crate::types::{BackendServer, LbAlgorithm};

use super::{least_connections_lb::LeastConnectionsStrategy, round_robin_lb::RoundRobinStrategy, weighted_round_robin_lb::WeightedRoundRobin};

pub struct SelectedLB {
    pub server: String,
    pub cleanup_fn: Box<dyn Fn() + Send + Sync>,
}

impl Drop for SelectedLB {
    fn drop(&mut self) {
        log::debug!("Running cleanup on LB: {}", self.server);
        (self.cleanup_fn)();
    }
}

#[async_trait::async_trait]
pub trait LoadBalancer: Send + Sync {
    async fn next(&self) -> Option<Arc<SelectedLB>>;
}

pub struct LoadBalancerFactory;

impl LoadBalancerFactory {
    pub fn create(algorithm: LbAlgorithm, server_backends: Vec<BackendServer>) -> Arc<dyn LoadBalancer> {
        match algorithm {
            LbAlgorithm::RoundRobin => 
                Arc::new(RoundRobinStrategy::new(server_backends.iter().map(|server| server.server.clone()).collect())),
            LbAlgorithm::LeastConnections =>
                Arc::new(LeastConnectionsStrategy::new(server_backends.iter().map(|server| server.server.clone()).collect())),
            LbAlgorithm::WeightedRoundRobin => Arc::new(WeightedRoundRobin::new(server_backends)),
        }
    }
}