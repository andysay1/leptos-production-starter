#[cfg(feature = "hydrate")]
pub fn main() {
    use app::SpaApp;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(|| leptos::prelude::view! { <SpaApp/> });
}

#[cfg(not(feature = "hydrate"))]
pub fn main() {}
