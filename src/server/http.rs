use std::{net::SocketAddr, sync::Arc};

use hyper::{body::Incoming, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpListener, time};

use crate::proxy_service::{gateway_body::GatewayBody, proxy_bridge::ProxyBridge};

pub async fn start_http_server(
    address: SocketAddr,
    proxy_bridge: &Arc<ProxyBridge>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tcp_listener = TcpListener::bind(&address).await?;

    loop {
        match tcp_listener.accept().await {
            Ok((stream, _)) => {
                stream.set_nodelay(true)?;

                let io = TokioIo::new(stream);

                let proxy_bridge = Arc::clone(&proxy_bridge);
                let service = Arc::new(service_fn(move |req| wrapper(req, proxy_bridge.clone())));

                tokio::spawn(async move {
                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        println!("Failed to serve the connection: {:?}", err);
                    }
                });
            }
            Err(e) => {
                log::warn!("Failed to accept connection {:?}", e);

                time::sleep(time::Duration::from_millis(10)).await;

                continue;
            }
        }
    }
}

async fn wrapper(
    req: Request<Incoming>,
    proxy_bridge: Arc<ProxyBridge>,
) -> Result<Response<GatewayBody>, hyper::Error> {
    Ok(proxy_bridge.determine(req).await)
}
