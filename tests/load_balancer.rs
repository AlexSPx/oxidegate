#[cfg(test)]
mod tests {
    use oxidegate::{
        types::BackendServer, LbAlgorithm, LeastConnectionsStrategy, LoadBalancer,
        LoadBalancerFactory, RoundRobinStrategy, WeightedRoundRobin,
    };
    use std::sync::{atomic::Ordering, Arc};

    #[tokio::test]
    async fn test_least_connections_strategy() {
        let servers = vec![
            "server1".to_string(),
            "server2".to_string(),
            "server3".to_string(),
        ];
        let strategy = LeastConnectionsStrategy::new(servers.clone());

        for (_, connections) in &strategy.servers {
            assert_eq!(connections.load(Ordering::Relaxed), 0);
        }

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server1");
        assert_eq!(strategy.servers[0].1.load(Ordering::Relaxed), 1);

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server2");
        assert_eq!(strategy.servers[1].1.load(Ordering::Relaxed), 1);

        match Arc::try_unwrap(selected) {
            Ok(lb) => (lb.cleanup_fn)(),
            Err(_) => todo!(),
        }

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server3");
        assert_eq!(strategy.servers[2].1.load(Ordering::Relaxed), 1);

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server1");
        assert_eq!(strategy.servers[0].1.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_round_robin_strategy() {
        let servers = vec![
            "server1".to_string(),
            "server2".to_string(),
            "server3".to_string(),
        ];
        let strategy = RoundRobinStrategy::new(servers.clone());

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server1");

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server2");

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server3");

        let selected = strategy.next().await.unwrap();
        assert_eq!(selected.server, "server1");
    }

    #[tokio::test]
    async fn test_weighted_round_robin_strategy() {
        let servers = vec![
            BackendServer {
                server: "server1".to_string(),
                weight: Some(1),
            },
            BackendServer {
                server: "server2".to_string(),
                weight: Some(2),
            },
            BackendServer {
                server: "server3".to_string(),
                weight: Some(3),
            },
        ];

        let strategy = WeightedRoundRobin::new(servers.clone());

        let expected_servers = vec![
            "server1", "server2", "server2", "server3", "server3", "server3",
        ];

        for expected in expected_servers {
            let selected = strategy.next().await.unwrap();
            assert_eq!(selected.server, expected);
        }
    }

    #[tokio::test]
    async fn test_load_balancer_factory() {
        let servers = vec![
            BackendServer {
                server: "server1".to_string(),
                weight: Some(1),
            },
            BackendServer {
                server: "server2".to_string(),
                weight: Some(1),
            },
        ];

        let round_robin_lb = LoadBalancerFactory::create(LbAlgorithm::RoundRobin, servers.clone());
        let least_connections_lb =
            LoadBalancerFactory::create(LbAlgorithm::LeastConnections, servers.clone());

        let selected = round_robin_lb.next().await.unwrap();
        assert_eq!(selected.server, "server1");

        let selected = round_robin_lb.next().await.unwrap();
        assert_eq!(selected.server, "server2");

        let selected = least_connections_lb.next().await.unwrap();
        assert_eq!(selected.server, "server1");

        let selected = least_connections_lb.next().await.unwrap();
        assert_eq!(selected.server, "server2");
    }
}
