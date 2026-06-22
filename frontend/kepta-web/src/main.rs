use kepta_ui::store::AppStore;
use leptos::prelude::*;
use leptos_router::{components::{Route, Router, Routes}, path};

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let store = AppStore::new();
    provide_context(store);

    view! {
        <Router>
            <Routes fallback=|| view! { <p>"Not found"</p> }>
                <Route path=path!("/") view=Dashboard />
                <Route path=path!("/login") view=Login />
                <Route path=path!("/households/:id") view=HouseholdView />
            </Routes>
        </Router>
    }
}

#[component]
fn Dashboard() -> impl IntoView {
    view! { <div class="p-8"><h1>"Dashboard"</h1></div> }
}

#[component]
fn Login() -> impl IntoView {
    view! { <div class="p-8"><h1>"Login"</h1></div> }
}

#[component]
fn HouseholdView() -> impl IntoView {
    view! { <div class="p-8"><h1>"Household"</h1></div> }
}
