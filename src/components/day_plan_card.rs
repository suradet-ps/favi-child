//! Renders one day's mixing plan as a card (AGENTS.md §4.4).
//!
//! Displays the tablet count `N`, diluent volume `V`, draw volume `D`, the
//! resulting actual mg per dose and its delta vs. the theoretical dose - the
//! delta is shown visibly for pharmacist QC, never hidden.

use leptos::prelude::*;

use crate::domain::MixingPlan;

/// Formats a tablet count: whole counts print without decimals ("1"), halves
/// print with one decimal ("0.5").
fn format_tablets(tablets: f64) -> String {
  if tablets.fract() == 0.0 {
    format!("{tablets:.0}")
  } else {
    format!("{tablets:.1}")
  }
}

/// Formats a volume with one decimal, e.g. `4.0`.
fn format_volume(ml: f64) -> String {
  format!("{ml:.1}")
}

/// A plan card for one regimen part (Day 1, or the shared Days 2-5 plan).
#[component]
pub fn DayPlanCard(title: String, subtitle: String, plan: MixingPlan) -> impl IntoView {
  let delta = if plan.delta_mg >= 0.0 {
    format!("+{:.1}", plan.delta_mg)
  } else {
    format!("{:.1}", plan.delta_mg)
  };
  view! {
      <section class="feature-card plan-card">
          <header class="plan-card-header">
              <h2 class="title-md">{title}</h2>
              <span class="badge-pill">{subtitle}</span>
          </header>
          <dl class="plan-rows">
              <div class="plan-row">
                  <dt>"ใช้ยาจำนวน"</dt>
                  <dd><span class="mono">{format_tablets(plan.tablets)}</span> " เม็ด (200 มก./เม็ด)"</dd>
              </div>
              <div class="plan-row">
                  <dt>"น้ำที่ใช้ละลาย"</dt>
                  <dd><span class="mono">{format_volume(plan.diluent_ml)}</span> " มล."</dd>
              </div>
              <div class="plan-row">
                  <dt>"ดูดยาให้ครั้งละ"</dt>
                  <dd><span class="mono">{format_volume(plan.draw_ml)}</span> " มล."</dd>
              </div>
              <div class="plan-row">
                  <dt>"รับประทาน"</dt>
                  <dd>"เช้า-เย็น (วันละ 2 ครั้ง)"</dd>
              </div>
          </dl>
          <footer class="plan-card-qc">
              <span class="mono">{format!("{:.1}", plan.delivered_mg)}</span>
              " มก./โดส ตามจริง "
              <span class="muted">
                  "(ค่าทางทฤษฎี " {format!("{:.1}", plan.dose_mg)} " มก. · ส่วนต่าง " {delta} " มก.)"
              </span>
          </footer>
      </section>
  }
}
