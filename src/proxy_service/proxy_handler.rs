use axum::{body::Body, response::IntoResponse};
use hyper::{Request, Response, StatusCode, Uri};
use hyper_util::{client::legacy::{connect::HttpConnector, Client}, rt::TokioExecutor};
use tokio::time::timeout;
use std::sync::Arc;

use crate::load_balancer::factory::LoadBalancer;

type HttpClient = Client<HttpConnector, Body>;

pub struct ProxyHandler {
    pub client: HttpClient,
    pub load_balancer: Arc<dyn LoadBalancer>,
}

impl ProxyHandler {
    pub fn new(balancer: Arc<dyn LoadBalancer>) -> Self {
        Self {
            client: Client::builder(TokioExecutor::new()).build(HttpConnector::new()),
            load_balancer: balancer,
        }
    }

    pub async fn handle(&self, req: Request<Body>) -> Response<Body> {
        let timeout_duration = std::time::Duration::from_secs(5);

        let selected_lb = self.load_balancer.next().await;


        match selected_lb {
            Some(lb ) => {
                let uri = self.build_backend_uri(&req, &lb.server);

                match timeout(timeout_duration, self.forward_request(req, uri)).await {
                    Ok(Ok(mut response)) => {
                        log::trace!("Response recieved: {:?}", response);
                        response.extensions_mut().insert(lb);

                        response
                    },
                    Ok(Err(e)) => {
                        Response::builder()
                            .status(e)
                            .body(Body::from("Bad request"))
                            .unwrap()
                    },
                    Err(_) => {
                        log::warn!("Request to backend timed out");

                        Response::builder()
                        .status(504)
                        .body(Body::from("Gateway Timeout"))
                        .unwrap()
                    }
                }
            },
            None => {
                Response::builder()
                    .status(503)
                    .body(Body::from("Service Unavailable"))
                    .unwrap()
            }
        }
    }

    fn build_backend_uri(&self, req: &Request<Body>, backend: &str) -> Uri {
        let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
        format!("{}{}", backend, path).parse().unwrap()
    }

    async fn forward_request(&self, mut req: Request<Body>, uri: Uri) -> Result<Response<Body>, StatusCode> {
        *req.uri_mut() = uri;
        Ok(self.client
            .request(req)
            .await
            .map_err(|e| {
                log::error!("Error forwarding request: {:?}", e);
                StatusCode::BAD_REQUEST
            })?
            .into_response())
    }
}

