//! FaviChild - pediatric Favipiravir suspension dosing calculator.
//!
//! A client-side (CSR/WASM) Leptos web app. The [`domain`] module holds all
//! clinical logic in pure Rust with zero Leptos dependencies; [`components`]
//! renders it (AGENTS.md §5).

pub mod components;
pub mod domain;
