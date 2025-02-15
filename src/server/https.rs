use std::{path::{Path, PathBuf}, pin::Pin, sync::Arc};

use axum::{extract::Request, Router};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio_rustls::{rustls::{pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer}, ServerConfig}, TlsAcceptor};
use tower_service::Service;

pub async fn start_https_server(mut app: Router, tcp_listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    let rustls_config =  rustls_server_config(
      PathBuf::from("certs/key.pem"),
        PathBuf::from("certs/cert.pem"),  
    )?;

    let tls_acceptor = TlsAcceptor::from(rustls_config);

    Pin::new(&mut app);
    loop {
        let tower_service: Router = app.clone();
        let tls_acceptor = tls_acceptor.clone();
        
        let (cnx, addr) = tcp_listener.accept().await.unwrap();
    
        tokio::spawn(async move {
            let Ok(stream) = tls_acceptor.accept(cnx).await else {
                log::error!("Error during tls handshake connection from {}", addr);
                return;
            };

            let stream = TokioIo::new(stream);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let ret = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await;

            if let Err(err) = ret {
                log::warn!("eError serving connection from {}: {}", addr, err);
            }
        });
    }
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