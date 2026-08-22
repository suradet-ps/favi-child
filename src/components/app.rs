//! Top-level component: wires the root weight signal to the derived plans
//! (AGENTS.md §6) and renders the input + result cards.

use leptos::prelude::*;

use crate::components::day_plan_card::DayPlanCard;
use crate::components::weight_input::WeightInput;
use crate::domain::{PlanError, RegimenPlan, plan_for_weight};

/// What to show in the results area for the current weight signal value.
#[derive(Clone)]
enum PlanView {
  /// No weight entered yet (empty input).
  Waiting,
  /// Weight is non-numeric / not finite / <= 0.
  InvalidWeight,
  /// Weight is plausible but no safe measurable plan exists.
  NoSafePlan,
  /// A full Day 1 + Days 2-5 regimen is ready.
  Ready(RegimenPlan),
}

/// Root component: weight input, derived plans, and result cards.
#[component]
pub fn App() -> impl IntoView {
  let weight_kg = RwSignal::new(None::<f64>);

  let plan_view = Signal::derive(move || match weight_kg.get() {
    None => PlanView::Waiting,
    Some(weight) => match plan_for_weight(weight) {
      Ok(plan) => PlanView::Ready(plan),
      Err(PlanError::InvalidWeight { .. }) => PlanView::InvalidWeight,
      Err(PlanError::NoSafePlan { .. }) => PlanView::NoSafePlan,
      Err(PlanError::InvalidDose { .. }) => {
        // Unreachable for a weight that passed plan_for_weight's own
        // validation; defensively render the invalid-weight state.
        PlanView::InvalidWeight
      }
    },
  });

  view! {
      <div class="page">
          <header class="page-header">
              <h1 class="display">FaviChild</h1>
              <p class="lede">
                  "วางแผนการเตรียมยาฟาวิพิราเวียร์แบบยาน้ำแขวนตะกอนจากยาเม็ด 200 มก."
              </p>
          </header>

          <WeightInput weight_kg=weight_kg />

          <section class="plan-area" aria-live="polite">
              {move || match plan_view.get() {
                  PlanView::Waiting => None,
                  PlanView::InvalidWeight => Some(
                      view! {
                          <p class="hint hint-error">"กรุณากรอกน้ำหนักตัวที่มากกว่า 0 (กก.)"</p>
                      }
                      .into_any(),
                  ),
                  PlanView::NoSafePlan => Some(
                      view! {
                          <p class="hint hint-error">
                              "ไม่พบแผนการเตรียมยาที่ปลอดภัยและวัดได้สำหรับน้ำหนักนี้"
                          </p>
                      }
                      .into_any(),
                  ),
                  PlanView::Ready(plan) => Some(
                      view! {
                          <div class="plan-grid">
                              <DayPlanCard
                                  title="วันที่ 1".to_string()
                                  subtitle="โดสยา 70 มก./กก./วัน".to_string()
                                  plan=plan.day1
                              />
                              <DayPlanCard
                                  title="วันที่ 2-5".to_string()
                                  subtitle="โดสยา 30 มก./กก./วัน".to_string()
                                  plan=plan.days2_5
                              />
                          </div>
                      }
                      .into_any(),
                  ),
              }}
          </section>

          <footer class="footer">
              <p>
                  "หมายเหตุ: ตัวเลขการคำนวณประกอบการตัดสินใจ โปรดยืนยันจากเภสัชกรก่อนนำไปใช้จริง"
              </p>
          </footer>
      </div>
  }
}
