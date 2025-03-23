use crate::types::BackendServer;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::factory::{LoadBalancer, SelectedLB};

pub struct WeightedRoundRobin {
    servers: Vec<(String, u32)>,
    current: AtomicUsize,
    total_weight: u32,
}

impl WeightedRoundRobin {
    pub fn new(servers: Vec<BackendServer>) -> Self {
        let total_weight = servers
            .iter()
            .map(|server| server.weight.unwrap_or(1))
            .sum();
        log::info!(
            "WeightedRoundRobinStrategy initialized with servers: {:?}, total_weight: {}",
            servers,
            total_weight
        );
        Self {
            servers: servers
                .into_iter()
                .map(|server| (server.server, server.weight.unwrap_or(1)))
                .collect(),
            current: AtomicUsize::new(0),
            total_weight,
        }
    }

    fn next_server(&self) -> Option<Arc<(&String, &u32)>> {
        let mut current = self.current.fetch_add(1, Ordering::Relaxed);
        current %= self.total_weight as usize;

        let mut cumulative_weight = 0;
        for (server, weight) in &self.servers {
            cumulative_weight += *weight;
            if current < cumulative_weight as usize {
                return Some(Arc::new((server, weight)));
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl LoadBalancer for WeightedRoundRobin {
    async fn next(&self) -> Option<Arc<SelectedLB>> {
        let (server, _) = *self.next_server()?;

        log::debug!("WeightedRoundRobin selected server: {}", server);

        let empty_fn = Box::new(move || {});

        Some(Arc::new(SelectedLB {
            server: server.clone(),
            cleanup_fn: empty_fn,
        }))
    }
}
