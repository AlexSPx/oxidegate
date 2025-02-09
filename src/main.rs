use std::{convert::Infallible, sync::Arc};

use config::load_config;
use hyper::{service::{make_service_fn, service_fn}, Body, Request, Response, Server};
use proxy::ProxyHandler;

mod types;
mod load_balancer;
mod proxy;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    env_logger::init();

    let config = match load_config().await {
        Ok(conf) => conf,
        Err(e) => {
            log::error!("Failed to load config: {}", e);
            return Err(e);
        },
    };

    let proxy_handlers: Arc<Vec<(types::Frontend, Arc<ProxyHandler>)>> = Arc::new(config.iter().map(|(frontend, balancer)| {
        let handler = ProxyHandler::new(balancer.to_owned());
        (frontend.clone(), Arc::new(handler))
    }).collect::<Vec<_>>());

    let http_addr = ([0, 0, 0, 0], 3000).into();

    let make_svc = make_service_fn(move |_| {
        let proxy_handlers = Arc::clone(&proxy_handlers);
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let proxy_handlers = Arc::clone(&proxy_handlers);
                async move {

                    let handler = proxy_handlers.iter().find(|(frontend, _)| {
                        frontend.path_prefix.iter().any(|prefix| req.uri().path().starts_with(prefix))
                    });
                    
                    match handler {
                        Some((_, handler)) => handler.handle(req).await,
                        None => {
                            Ok(Response::builder()
                                .status(404)
                                .body(Body::from("Not Found"))
                                .unwrap())
                        }
                    }
                }
            }))
        }
    });

    let http_server = Server::bind(&http_addr).serve(make_svc);

    log::info!("Server running on http://{}", http_addr);

    let graceful = http_server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        log::error!("Server error: {}", e);
    }

    log::info!("Server stopped.");

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sigint.recv() => log::info!("Received SIGINT (Ctrl+C)"),
        _ = sigterm.recv() => log::info!("Received SIGTERM (Termination)"),
    }

    log::info!("Shutdown signal received. Exiting...");

    std::process::exit(0);
}
