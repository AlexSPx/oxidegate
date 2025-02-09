use hyper::{Body, Request, Response, Client, Uri};
use hyper::client::HttpConnector;
use tokio::time::timeout;
use std::sync::Arc;

use crate::load_balancer::factory::LoadBalancer;

type HttpClient = Client<HttpConnector>;

pub struct ProxyHandler {
    client: HttpClient,
    load_balancer: Box<Arc<dyn LoadBalancer>>,
}

impl ProxyHandler {
    pub fn new(balancer: Box<Arc<dyn LoadBalancer>>) -> Self {
        Self {
            client: Client::new(),
            load_balancer: balancer,
        }
    }

    pub async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let timeout_duration = std::time::Duration::from_secs(5);

        let backend_uri = self.load_balancer.next().await
            .and_then(|server| self.build_backend_uri(&req, &server))
            .and_then(|uri| uri.parse().ok());

        match backend_uri {
            Some(uri) => {
                match timeout(timeout_duration, self.forward_request(req, uri)).await {
                    Ok(Ok(response)) => {
                        log::trace!("Response recieved: {:?}", response);
                        Ok(response)
                    },
                    Ok(Err(e)) => {
                        log::error!("Error forwarding request: {:?}", e);
                        Err(e)
                    },
                    Err(_) => {
                        log::warn!("Request to backend timed out");

                        Ok(Response::builder()
                        .status(504)
                        .body(Body::from("Gateway Timeout"))
                        .unwrap())
                    }
                }
            },
            None => {
                Ok(Response::builder()
                    .status(503)
                    .body(Body::from("Service Unavailable"))
                    .unwrap())
            }
        }
    }

    fn build_backend_uri(&self, req: &Request<Body>, backend: &str) -> Option<String> {
        let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
        Some(format!("{}{}", backend, path))
    }

    async fn forward_request(&self, mut req: Request<Body>, uri: Uri) -> Result<Response<Body>, hyper::Error> {
        *req.uri_mut() = uri;
        self.client.request(req).await
    }
}

