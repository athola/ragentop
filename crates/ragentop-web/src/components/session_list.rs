//! Session list component displaying all agent sessions.

use leptos::prelude::*;

/// Props for a session item in the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionItem {
    pub id: String,
    pub agent_type: String,
    pub status: String,
    pub working_dir: Option<String>,
}

/// Component displaying a list of agent sessions.
#[expect(clippy::must_use_candidate)] // Leptos components are used via view! macro
#[component]
pub fn SessionList(
    /// List of sessions to display.
    sessions: Signal<Vec<SessionItem>>,
    /// Callback when a session is selected.
    #[prop(into)]
    on_select: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="session-list">
            <h3>"Sessions"</h3>
            <ul class="session-list__items">
                <For
                    each=move || sessions.get()
                    key=|session| session.id.clone()
                    children=move |session| {
                        let id = session.id.clone();
                        let on_click_id = id.clone();
                        let on_select = on_select;
                        view! {
                            <li
                                class="session-list__item"
                                on:click=move |_| on_select.run(on_click_id.clone())
                            >
                                <span class="session-list__agent">{session.agent_type}</span>
                                <span class="session-list__id">{id}</span>
                                <span class=format!("session-list__status session-list__status--{}", session.status)>
                                    {session.status.clone()}
                                </span>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}
