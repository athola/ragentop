//! Session detail component showing detailed session information.

use leptos::prelude::*;

/// Detailed session information for display.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SessionDetails {
    pub id: String,
    pub agent_type: String,
    pub model: Option<String>,
    pub status: String,
    pub working_dir: Option<String>,
    pub token_count: u64,
    pub cost_usd: Option<f64>,
    pub command_count: u64,
}

/// Component displaying detailed information about a selected session.
#[expect(clippy::must_use_candidate)] // Leptos components are used via view! macro
#[component]
pub fn SessionDetail(
    /// The session details to display, None if no session selected.
    session: Signal<Option<SessionDetails>>,
) -> impl IntoView {
    view! {
        <div class="session-detail">
            {move || match session.get() {
                Some(details) => view! {
                    <div class="session-detail__content">
                        <h3>"Session Details"</h3>
                        <dl class="session-detail__info">
                            <dt>"ID"</dt>
                            <dd>{details.id}</dd>

                            <dt>"Agent"</dt>
                            <dd>{details.agent_type}</dd>

                            <dt>"Model"</dt>
                            <dd>{details.model.unwrap_or_else(|| "N/A".to_owned())}</dd>

                            <dt>"Status"</dt>
                            <dd class=format!("session-detail__status--{}", details.status)>
                                {details.status.clone()}
                            </dd>

                            <dt>"Working Directory"</dt>
                            <dd>{details.working_dir.unwrap_or_else(|| "N/A".to_owned())}</dd>

                            <dt>"Tokens"</dt>
                            <dd>{details.token_count.to_string()}</dd>

                            <dt>"Cost"</dt>
                            <dd>{
                                details.cost_usd
                                    .map_or_else(|| "N/A".to_owned(), |c| format!("${c:.4}"))
                            }</dd>

                            <dt>"Commands"</dt>
                            <dd>{details.command_count.to_string()}</dd>
                        </dl>
                    </div>
                }.into_any(),
                None => view! {
                    <div class="session-detail__empty">
                        <p>"Select a session to view details"</p>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
