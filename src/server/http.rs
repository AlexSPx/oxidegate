use axum::Router;
use tokio::net::TcpListener;

pub async fn start_http_server(app: Router, tcp_listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {

    axum::serve(tcp_listener, app).await?;

    Ok(())
}