//! Main dashboard layout component.

use leptos::prelude::*;

use super::session_detail::{SessionDetail, SessionDetails};
use super::session_list::{SessionItem, SessionList};

/// Main dashboard component with session list and detail panels.
#[expect(clippy::must_use_candidate)] // Leptos components are used via view! macro
#[component]
pub fn Dashboard() -> impl IntoView {
    // Mock data for initial development
    let initial_sessions = vec![
        SessionItem {
            id: "session-001".to_owned(),
            agent_type: "claude".to_owned(),
            status: "active".to_owned(),
            working_dir: Some("/home/user/project".to_owned()),
        },
        SessionItem {
            id: "session-002".to_owned(),
            agent_type: "codex".to_owned(),
            status: "idle".to_owned(),
            working_dir: Some("/home/user/other".to_owned()),
        },
    ];

    let (sessions, _set_sessions) = signal(initial_sessions);
    let (selected_id, set_selected_id) = signal::<Option<String>>(None);

    // Derive selected session details from selected_id
    let selected_session = Memo::new(move |_| {
        selected_id.get().and_then(|id| {
            sessions
                .get()
                .into_iter()
                .find(|s| s.id == id)
                .map(|s| SessionDetails {
                    id: s.id,
                    agent_type: s.agent_type,
                    model: Some("claude-opus-4-5-20251101".to_owned()),
                    status: s.status,
                    working_dir: s.working_dir,
                    token_count: 15000,
                    cost_usd: Some(0.045),
                    command_count: 42,
                })
        })
    });

    let on_select = Callback::new(move |id: String| {
        set_selected_id.set(Some(id));
    });

    view! {
        <div class="dashboard">
            <header class="dashboard__header">
                <h2>"Agent Monitor"</h2>
                <span class="dashboard__session-count">
                    {move || format!("{} sessions", sessions.get().len())}
                </span>
            </header>
            <div class="dashboard__content">
                <aside class="dashboard__sidebar">
                    <SessionList sessions=sessions.into() on_select=on_select />
                </aside>
                <main class="dashboard__main">
                    <SessionDetail session=selected_session.into() />
                </main>
            </div>
        </div>
    }
}
