#[cfg(feature = "hydrate")]
pub fn main() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| leptos::view! { <App/> });
}

#[cfg(not(feature = "hydrate"))]
pub fn main() {}
