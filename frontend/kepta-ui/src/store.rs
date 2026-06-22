use kepta_types::{auth::UserProfile, households::Household};
use leptos::prelude::*;

/// Global reactive store — all in-memory, no IndexedDB.
#[derive(Clone)]
pub struct AppStore {
    pub current_user: RwSignal<Option<UserProfile>>,
    pub households: RwSignal<Vec<Household>>,
    pub access_token: RwSignal<Option<String>>,
}

impl AppStore {
    pub fn new() -> Self {
        Self {
            current_user: RwSignal::new(None),
            households: RwSignal::new(vec![]),
            access_token: RwSignal::new(None),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.access_token.get().is_some()
    }
}
