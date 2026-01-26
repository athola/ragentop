//! Main application loop.

use crate::state::AppState;

pub struct App {
    pub state: AppState,
}

impl App {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
