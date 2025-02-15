use std::sync::Arc;

use axum::{extract::{Request, State}, middleware::map_response, response::Response, routing::any, Router};
use tokio::net::TcpListener;
use tower::ServiceBuilder;

use crate::{load_balancer::factory::SelectedLB, proxy_service::proxy_bridge::ProxyBridge, types::ServerSettings};

use super::{http::start_http_server, https::start_https_server};

pub struct ServerManager {
    settings: ServerSettings,
    proxy_bridge: Arc<ProxyBridge>
}

impl ServerManager {
    pub fn new(settings: ServerSettings, proxy_bridge: Arc<ProxyBridge>) -> Self {
        Self {
            settings,
            proxy_bridge
        }
    }

    pub async fn start_server(&self) -> Result<(), Box<dyn std::error::Error>> {

        let app = Router::new().route("/{*wildcard}",any(handle_routes)
            .with_state(self.proxy_bridge.clone())
            .layer(ServiceBuilder::new().layer(map_response(post_request))));

            let bind = format!("{}:{}", "0.0.0.0", self.settings.port);
            let tcp_listener = TcpListener::bind(bind).await?;

            log::info!("Server started on port: {}", self.settings.port);

        if self.settings.enable_https {
            start_https_server(app, tcp_listener).await
        } else {
            start_http_server(app, tcp_listener).await
        }
    }
}

async fn handle_routes(State(proxy_bridge): State<Arc<ProxyBridge>>, req: Request) -> Response {
    log::info!("Request recieved: {:?}", req);

    proxy_bridge.determine(req).await
}

async fn post_request(mut response: Response) -> Response {

    if let Some(selected_lb) = response.extensions_mut().remove::<Arc<SelectedLB>>() {
        log::debug!("Cleaning up selected load balancer: {:?}", selected_lb.server);

        match Arc::try_unwrap(selected_lb) {
            Ok(lb) => (lb.cleanup_fn)(),
            Err(_) => log::warn!("Could not clean up load balancer: More than one reference exists")
        }
    }

    response
}
