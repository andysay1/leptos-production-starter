#[cfg(feature = "hydrate")]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_islands();
}

#[cfg(not(feature = "hydrate"))]
pub fn main() {}
