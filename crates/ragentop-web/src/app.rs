//! Leptos App component for ragentop web dashboard.

use leptos::prelude::*;

use crate::components::Dashboard;

/// Main application component.
#[expect(clippy::must_use_candidate)] // Leptos components are used via view! macro
#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
            <h1>"ragentop"</h1>
            <p>"AI Agent Monitor - Web Dashboard"</p>
            <Dashboard />
        </main>
    }
}

/// Render the app to HTML string for SSR.
#[inline]
#[must_use]
pub fn render_app() -> String {
    use leptos::prelude::RenderHtml as _;
    let owner = Owner::new();
    let html = owner.with(|| App().to_html());
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>ragentop</title>
</head>
<body>
{html}
</body>
</html>"#
    )
}
