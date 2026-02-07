//! Web server startup and browser integration.

use axum::{response::Html, routing::get, Router};

use crate::app::render_app;

/// Start the web dashboard server and open a browser.
///
/// Binds to the given host and port, attempts to open the default browser,
/// and serves the dashboard until the process is terminated.
///
/// # Errors
///
/// Returns an error if the TCP listener cannot bind or the server fails.
pub async fn serve(host: [u8; 4], port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::net::SocketAddr::from((host, port));

    let app = Router::new().route("/", get(index));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let url = format!("http://{addr}");
    eprintln!("Web dashboard: {url}");

    try_open_browser(&url);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<String> {
    Html(render_app())
}

/// Try to open the URL in the system default browser, falling back to firefox.
///
/// Tries commands in order: `xdg-open` (Linux default), `wslview` (WSL),
/// then `firefox`. Prints a message if none succeed.
fn try_open_browser(url: &str) {
    use std::process::{Command, Stdio};

    // Try system default browser, then WSL browser bridge
    for cmd in &["xdg-open", "wslview"] {
        if Command::new(cmd)
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
        {
            return;
        }
    }

    // Fall back to firefox directly
    match Command::new("firefox")
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Could not open browser automatically: {e}");
            eprintln!("Please open {url} manually");
        }
    }
}
