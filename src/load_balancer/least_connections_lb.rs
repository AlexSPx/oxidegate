use std::sync::{atomic::AtomicUsize, Arc};

use super::factory::{LoadBalancer, SelectedLB};

pub struct LeastConnectionsStrategy {
    pub servers: Vec<(String, Arc<AtomicUsize>)>,
}

impl LeastConnectionsStrategy {
    pub fn new(servers: Vec<String>) -> Self {
        log::info!(
            "LeastConnectionsStrategy initialized with servers: {:?}",
            servers
        );
        Self {
            servers: servers
                .into_iter()
                .map(|s| (s, Arc::new(AtomicUsize::new(0))))
                .collect(),
        }
    }
}

#[async_trait::async_trait]
impl LoadBalancer for LeastConnectionsStrategy {
    async fn next(&self) -> Option<Arc<SelectedLB>> {
        let (server, connections) = self.servers.iter().min_by_key(|(_, connections)| {
            connections.load(std::sync::atomic::Ordering::Relaxed)
        })?;

        log::debug!(
            "LeastConnectionsStrategy selected server: {}, current connections: {}",
            server,
            connections.load(std::sync::atomic::Ordering::Relaxed)
        );

        connections.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let connections_clone = Arc::clone(connections);

        let clean_up_fn = Box::new(move || {
            connections_clone.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        });

        Some(Arc::new(SelectedLB {
            server: server.clone(),
            cleanup_fn: clean_up_fn,
        }))
    }
}
