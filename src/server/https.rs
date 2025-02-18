use std::{net::{Ipv4Addr, SocketAddr}, path::{Path, PathBuf}, sync::Arc};

use hyper::{body::Incoming, service::service_fn, Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio_rustls::{rustls::{pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer}, ServerConfig}, TlsAcceptor};

use crate::proxy_service::{gateway_body::GatewayBody, proxy_bridge::ProxyBridge};

pub async fn start_https_server(address: SocketAddr, proxy_bridge: Arc<ProxyBridge>) -> Result<(), Box<dyn std::error::Error>> {
    let rustls_config =  rustls_server_config(
      PathBuf::from("certs/key.pem"),
        PathBuf::from("certs/cert.pem"),  
    )?;


    let tcp_listener = TcpListener::bind(&address).await?;
    let tls_acceptor = TlsAcceptor::from(rustls_config);

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;

        let tls_acceptor = tls_acceptor.clone();
        let proxy_bridge = Arc::clone(&proxy_bridge);

        let service = Arc::new(service_fn(move |req| wrapper(req, proxy_bridge.clone())));
        tokio::spawn(async move {
            let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Error during TLS handshake: {:?}", e);
                    return;
                }
            };

            let _ = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(TokioIo::new(tls_stream), service)
                .await;
        });
    }
}

async fn wrapper(req: Request<Incoming>, proxy_bridge: Arc<ProxyBridge>) -> Result<Response<GatewayBody>, hyper::Error> {
    Ok(proxy_bridge.determine(req).await)
}

fn rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Result<Arc<ServerConfig>, Box<dyn std::error::Error>> {
    log::info!("Loading key and cert from, key: {:?}, cert: {:?}", key.as_ref(), cert.as_ref());
    
    let key = PrivateKeyDer::from_pem_file(key)?;

    let certs = CertificateDer::pem_file_iter(cert)?
        .map(|cert| cert.unwrap())
        .collect::<Vec<_>>();

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs,key)
        .expect("Bad certificate or key");

        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    log::info!("TLS server config loaded");

    Ok(Arc::new(config))
}