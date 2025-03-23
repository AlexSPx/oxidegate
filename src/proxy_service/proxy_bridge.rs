use std::sync::Arc;

use crate::types::Frontend;
use hyper::{body::Incoming, Request, Response, StatusCode};

use super::{gateway_body::GatewayBody, proxy_handler::ProxyHandler};

pub struct ProxyBridge {
    proxy_handlers: Arc<Vec<(Frontend, Arc<ProxyHandler>)>>,
}

impl ProxyBridge {
    pub fn new(proxy_handlers: Arc<Vec<(Frontend, Arc<ProxyHandler>)>>) -> Self {
        Self { proxy_handlers }
    }

    pub async fn determine(&self, req: Request<Incoming>) -> Response<GatewayBody> {
        log::info!("Request recieced with path: {:?}", req.uri().path());

        let path = req.uri().path();
        let handler = self.proxy_handlers.iter().find(|(frontend, _)| {
            frontend.path_prefix.iter().any(|prefix| {
                if prefix == "/*" {
                    true
                } else if prefix.ends_with("/*") {
                    let trimmed_prefix = &prefix[..prefix.len() - 1];
                    path.starts_with(trimmed_prefix)
                } else {
                    path == prefix
                }
            })
        });

        log::debug!("Handler found: {:?}", handler.is_some());

        match handler {
            Some((_, handler)) => handler.handle(req).await,
            None => Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(GatewayBody::Empty)
                .unwrap(),
        }
    }
}
