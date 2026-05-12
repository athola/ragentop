//! Session detail component showing detailed session information.

use leptos::prelude::*;
use ragentop_core::UsdMicros;

/// Detailed session information for display.
///
/// `cost_usd` is [`UsdMicros`] so it flows from core's `SessionMetrics`
/// without an `f64` round-trip at the API boundary. Display formatting
/// happens in the component view via [`UsdMicros::Display`], which
/// already pads to six fractional digits.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SessionDetails {
    pub id: String,
    pub agent_type: String,
    pub model: Option<String>,
    pub status: String,
    pub working_dir: Option<String>,
    pub token_count: u64,
    pub cost_usd: Option<UsdMicros>,
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
                                    .map_or_else(|| "N/A".to_owned(), |c| c.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_usd_field_is_typed_usd_micros() {
        // Compile-time guarantee: the field accepts UsdMicros, not raw f64.
        // Reverting to Option<f64> would break this test and any caller
        // that constructs SessionDetails from core's SessionMetrics.
        let details = SessionDetails {
            id: "s-1".to_owned(),
            agent_type: "claude".to_owned(),
            model: None,
            status: "active".to_owned(),
            working_dir: None,
            token_count: 0,
            cost_usd: Some(UsdMicros::from_dollars(0.045)),
            command_count: 0,
        };
        assert_eq!(
            details.cost_usd,
            Some(UsdMicros::from_micros(45_000)),
            "0.045 dollars = 45_000 micros (exact, no f64 drift)"
        );
    }

    #[test]
    fn cost_usd_display_uses_usd_micros_format() {
        let details = SessionDetails {
            cost_usd: Some(UsdMicros::from_dollars(0.045)),
            ..SessionDetails::default()
        };
        // UsdMicros::Display pads to six fractional digits.
        let rendered = details
            .cost_usd
            .map_or_else(|| "N/A".to_owned(), |c| c.to_string());
        assert_eq!(rendered, "$0.045000");
    }
}
