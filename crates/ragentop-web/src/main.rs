//! Web server for ragentop dashboard.

use axum::{response::Html, routing::get, Router};
use ragentop_web::app::render_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host: [u8; 4] = std::env::var("RAGENTOP_HOST")
        .ok()
        .and_then(|s| s.parse::<std::net::Ipv4Addr>().ok())
        .map_or([127, 0, 0, 1], |ip| ip.octets());
    let port: u16 = std::env::var("RAGENTOP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = std::net::SocketAddr::from((host, port));

    let app = Router::new().route("/", get(index));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("listening on http://{}", &addr);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<String> {
    Html(render_app())
}
