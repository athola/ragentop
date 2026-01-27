//! Web server for ragentop dashboard.

use axum::{response::Html, routing::get, Router};
use ragentop_web::app::render_app;

#[tokio::main]
async fn main() {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));

    let app = Router::new().route("/", get(index));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening on http://{}", &addr);
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> Html<String> {
    Html(render_app())
}
