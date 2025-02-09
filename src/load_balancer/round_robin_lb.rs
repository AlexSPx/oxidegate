use std::sync::atomic::{AtomicUsize, Ordering};

use super::factory::LoadBalancer;

pub struct RoundRobinStrategy {
    servers: Vec<String>,
    current: AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new(servers: Vec<String>) -> Self {
        log::info!("RoundRobinStrategy initialized with servers: {:?}", servers);
        Self {
            servers,
            current: AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for RoundRobinStrategy {
    async fn next(&self) -> Option<String> {
        let current = self.current.fetch_add(1, Ordering::Relaxed);
        let server = self.servers.get(current % self.servers.len())?;

        log::debug!("RoundRobinStrategy selected server: {}", server);

        Some(server.clone())
    }
}