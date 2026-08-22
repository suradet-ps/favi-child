//! Number input for the child's body weight (kg).
//!
//! Parses free-form input into an `Option<f64>` written into the root
//! `weight_kg` signal (AGENTS.md §6): `None` for empty / unparseable input,
//! `Some(w)` for a parseable number (including invalid ranges - validation
//! is the domain's job, not this component's).

use leptos::prelude::*;

/// Weight input labelled "น้ำหนักตัว (กก.)".
///
/// `weight_kg` is the single source of truth at the app root (§6); every
/// keystroke re-parses the input and writes the result into it.
#[component]
pub fn WeightInput(weight_kg: RwSignal<Option<f64>>) -> impl IntoView {
    view! {
        <label class="text-input" for="weight-input">
            <span class="caption-uppercase">"น้ำหนักตัว"</span>
            <span class="text-input-row">
                <input
                    id="weight-input"
                    type="number"
                    inputmode="decimal"
                    min="0"
                    step="0.1"
                    placeholder="เช่น 4.6"
                    on:input=move |ev| {
                        let raw = event_target_value(&ev);
                        let parsed = raw.trim().parse::<f64>().ok();
                        weight_kg.set(parsed);
                    }
                />
                <span class="unit">"กก."</span>
            </span>
        </label>
    }
}
