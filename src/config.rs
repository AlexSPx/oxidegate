use std::sync::Arc;

use tokio::fs;

use crate::{load_balancer::factory::{LoadBalancer, LoadBalancerFactory}, types::{Backend, Frontend}};

use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    frontends: Vec<Frontend>,
    backends: Vec<Backend>,
}

pub async fn load_config() -> Result<Vec<(Frontend, Box<Arc<dyn LoadBalancer>>)>, Box<dyn std::error::Error>> {
    let file_path = match std::env::var("CONFIG_FILE") {
        Ok(path) => path,
        Err(_) => "config.yaml".to_string(),
    };
    
    log::debug!("Loading config from: {}", file_path);

    let yaml_content = fs::read_to_string(file_path).await?;
    
    let config: Config = serde_yaml::from_str(&yaml_content)?;

    let connections: Vec<(Frontend, Box<Arc<dyn LoadBalancer>>)> = config.frontends.iter().map(|frontend| {
        let backend = config.backends.iter().find(|backend| backend.name == frontend.backend).unwrap();
        let balancer = LoadBalancerFactory::create(backend.lb_algorithm, backend.servers.clone());

        (frontend.clone(), Box::new(balancer))
    }).collect::<Vec<_>>();
    
    Ok(connections)
}