use hyper::{body::Incoming, Request, Response, StatusCode, Uri};
use hyper_rustls::{ConfigBuilderExt, HttpsConnector};
use hyper_util::{client::legacy::{connect::HttpConnector, Client}, rt::TokioExecutor};
use tokio::time::timeout;
use tokio_rustls::rustls;
use std::sync::Arc;

use crate::load_balancer::factory::LoadBalancer;

use super::gateway_body::GatewayBody;

type HttpClient = Client<HttpsConnector<HttpConnector>, GatewayBody>;

pub struct ProxyHandler {
    pub client: HttpClient,
    pub load_balancer: Arc<dyn LoadBalancer>,
}

impl ProxyHandler {
    pub fn new(balancer: Arc<dyn LoadBalancer>) -> Self {
        let c = rustls::ClientConfig::builder()
            .with_native_roots().unwrap()
            .with_no_client_auth();

        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(c)
            .https_or_http()
            .enable_http1()
            .build();

        let client: Client<_, GatewayBody> = Client::builder(TokioExecutor::new()).build(https);

        Self {
            client,
            load_balancer: balancer,
        }
    }

    pub async fn handle(&self, mut req: Request<Incoming>) -> Response<GatewayBody> {

        let selected_lb = self.load_balancer.next().await;
        
        match selected_lb {
            Some(backend) => {
                let backend_uri = self.build_backend_uri(&req, &backend.server);
                log::debug!("Proxying request to: {}", backend_uri);

                self.proxy_request(req, &backend_uri).await
            },
            None => Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(GatewayBody::Empty)
                .unwrap(),
        }
    }

    fn build_backend_uri(&self, req: &Request<Incoming>, backend: &str) -> Uri {
        let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
        format!("{}{}", backend, path).parse().unwrap()
    }

    async fn proxy_request(&self ,req: Request<Incoming>, backend_uri: &Uri) -> Response<GatewayBody> {
        let timeout_duration = std::time::Duration::from_secs(5);

        let headers = req.headers().clone();
        let method = req.method().clone();
        let uri = backend_uri;
        let body = req.into_body();

        let new_req = Request::builder()
            .method(method)
            .uri(uri)
            .body(GatewayBody::Incomming(body))
            .map_err(|e| {
                log::error!("Failed to build request: {}", e);
                e
            });

        let new_req = match new_req {
            Ok(mut req) => {
                *req.headers_mut() = headers;
                req
            },
            Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(GatewayBody::Empty)
                .unwrap();
            }
        };

        match timeout(timeout_duration, self.client.request(new_req)).await {
            Ok(Ok(res)) => {
                let (parts, body) = res.into_parts();
                let body = GatewayBody::Incomming(body);
                Response::from_parts(parts, body)
            },
            Ok(Err(e)) => {
                log::warn!("Error proxying request: {}", e);
                log::debug!("Connection info: {:?}", e.connect_info());
                
                Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(GatewayBody::Empty)
                    .unwrap()
            },
            Err(e) => {
                log::warn!("Request timed out: {}", e);

                Response::builder()
                    .status(StatusCode::BAD_GATEWAY)
                    .body(GatewayBody::Empty)
                    .unwrap()
            },
        }
    }
}