//! FaviChild - Leptos CSR entry point (AGENTS.md §3, §5).

use favi_child::components::app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::prelude::mount_to_body(App);
}
