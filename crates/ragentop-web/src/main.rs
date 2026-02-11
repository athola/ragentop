//! Web server binary for ragentop dashboard.

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

    ragentop_web::serve(host, port).await
}
