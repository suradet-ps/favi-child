//! Leptos UI components - rendering only (AGENTS.md §5).
//!
//! These components never contain dosing math; they call into
//! [`crate::domain`] and render the results. Visual tokens follow
//! `docs/DESIGN.md`.

pub mod app;
pub mod day_plan_card;
pub mod weight_input;
