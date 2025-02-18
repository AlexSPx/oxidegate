use std::sync::Arc;
use config::load_config;
use load_balancer::factory::{LoadBalancer, LoadBalancerFactory};
use proxy_service::{proxy_bridge::ProxyBridge, proxy_handler::ProxyHandler};
use server::server_manager::ServerManager;
use types::Frontend;

mod types;
mod load_balancer;
mod proxy_service;
mod config;
mod server; 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    env_logger::init();

    let config = match load_config().await {
        Ok(conf) => conf,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(e);
        },
    };

    let connections: Vec<(Frontend, Arc<dyn LoadBalancer>)> = config.frontends.iter().map(|frontend| {
        let backend = config.backends.iter().find(|backend| backend.name == frontend.backend).unwrap();
        let balancer = LoadBalancerFactory::create(backend.lb_algorithm, backend.servers.clone());

        (frontend.clone(), balancer)
    }).collect::<Vec<_>>();

    let proxy_handlers: Arc<Vec<(types::Frontend, Arc<ProxyHandler>)>> = Arc::new(connections.iter().map(|(frontend, balancer)| {
        let handler = ProxyHandler::new(balancer.to_owned());
        (frontend.clone(), Arc::new(handler))
    }).collect::<Vec<_>>());

    let proxy_bridge: Arc<ProxyBridge> = Arc::new(ProxyBridge::new(proxy_handlers));

    let server_manager = ServerManager::new(config.server, proxy_bridge);

    server_manager.start_server().await
}

