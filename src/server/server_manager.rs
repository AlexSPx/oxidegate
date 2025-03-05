use std::{net::{Ipv4Addr, SocketAddr}, sync::Arc};

use crate::{proxy_service::proxy_bridge::ProxyBridge, types::ServerSettings};

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

        let address: SocketAddr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), self.settings.port);

        if self.settings.enable_https {
            start_https_server(address,
                 Arc::clone(&self.proxy_bridge),
                    self.settings.key_path.as_ref().unwrap(),
                    self.settings.cert_path.as_ref().unwrap()
                ).await
        } else {
            start_http_server(address, Arc::clone(&self.proxy_bridge)).await
        }
    }
}
