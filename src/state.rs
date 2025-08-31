use std::sync::Arc;

/// Application state that can be shared across handlers
#[derive(Clone)]
pub struct AppState {
    // Add any shared state here, like database connections, configuration, etc.
}

impl Default for AppState {
    fn default() -> Self {
        Self {}
    }
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}
