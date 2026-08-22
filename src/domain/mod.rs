//! Clinical domain logic - pure Rust, **zero Leptos dependencies** (§5).
//!
//! All dosing math and protocol rules from AGENTS.md §4 live here and are
//! independently unit-testable. UI components must never contain dosing math;
//! they call into this module and render the results.

pub mod constants;
pub mod plan;
pub mod regimen;

pub use constants::*;
pub use plan::{MixingPlan, PlanError, RegimenPlan, plan_for_dose, plan_for_weight};
pub use regimen::{RegimenDay, per_administration_mg};
