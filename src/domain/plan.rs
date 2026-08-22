//! Reconstitution / administration-plan search (AGENTS.md §4.3).
//!
//! The required per-dose mg rarely corresponds to a physically measurable
//! fraction of a 200 mg tablet, so the plan is **not** "give X tablets" but
//! "dissolve N tablets in V mL of water, then draw and administer D mL".
//!
//! ```text
//! concentration_mg_per_ml = (N * 200) / V
//! D = dose_mg / concentration_mg_per_ml
//! ```
//!
//! # Selection criteria (searched over candidate `(N, V)` pairs)
//!
//! 1. **Diluent preference** - `V` comes from the two-size set `{5, 10}` mL;
//!    the 5 mL size is always tried first, with 10 mL used only when no valid
//!    plan fits in 5 mL (user-confirmed clinical choice, pending เวช sign-off).
//! 2. **Correctness** - the rounded draw volume satisfies `D <= V`.
//! 3. **Measurability** - `D` is a multiple of 0.5 mL (syringe graduation)
//!    and at least 1.0 mL.
//! 4. **Roundness** - tie-break: a whole-mL `D` is preferred over a half-mL
//!    `D` when both are within the rounding-error tolerance
//!    (see [`ROUNDING_TOLERANCE_ML`], value pending pharmacist confirmation).
//! 5. **Economy** - the smallest tablet count `N` wins.
//!
//! # Termination
//!
//! The search enumerates `N` from 0.5 up to the smallest 0.5-multiple that is
//! `>= dose / 200`. That boundary is provably valid: with `V = 5 mL` the
//! exact draw volume there is at most 5 mL (and at least ~0.83 mL for
//! `dose_mg >= 20`), so a candidate always exists for `dose_mg >= 20`. Below
//! that the explicit [`PlanError::NoSafePlan`] rejection is returned - never
//! a silently-invalid draw volume (§4.5).

use std::fmt;

use super::constants::{
  DILUENT_VOLUMES_ML, DRAW_VOLUME_GRADUATION_ML, MIN_DRAW_VOLUME_ML, ROUNDING_TOLERANCE_ML,
  TABLET_FRACTION_STEP, TABLET_STRENGTH_MG,
};
use super::regimen::{RegimenDay, per_administration_mg};

/// A fully-specified mixing plan for one day's per-dose administration (§4.4).
#[derive(Debug, Clone, PartialEq)]
pub struct MixingPlan {
  /// Theoretical per-dose mg this plan targets (§4.2).
  pub dose_mg: f64,
  /// Tablets to crush/dissolve, a multiple of [`TABLET_FRACTION_STEP`].
  pub tablets: f64,
  /// Diluent (water) volume in mL.
  pub diluent_ml: f64,
  /// Draw volume per administration in mL, a multiple of 0.5 mL.
  pub draw_ml: f64,
  /// Resulting concentration of the mixed suspension, mg per mL.
  pub concentration_mg_per_ml: f64,
  /// Actual mg delivered per dose (`draw_ml * concentration`) - for pharmacist QC.
  pub delivered_mg: f64,
  /// `delivered_mg - dose_mg` - the rounding delta, shown visibly, not hidden (§4.4).
  pub delta_mg: f64,
}

/// The complete regimen: one plan for Day 1, one shared plan for Days 2-5.
///
/// Days 2-5 reuse **exactly one** mixing plan (§4.3 rule 5): the per-dose mg
/// is constant across those four days, so the plan is computed once and
/// reused - the caregiver mixes once and measures from the same suspension.
#[derive(Debug, Clone, PartialEq)]
pub struct RegimenPlan {
  /// Day 1 plan (dose = 35 mg/kg).
  pub day1: MixingPlan,
  /// Shared Days 2-5 plan (dose = 15 mg/kg), identical for all four days.
  pub days2_5: MixingPlan,
}

/// Reasons [`plan_for_weight`] cannot produce a regimen plan (§4.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanError {
  /// Weight is non-numeric, not finite, or `<= 0`.
  InvalidWeight { weight_kg: f64 },
  /// `dose_mg` passed to [`plan_for_dose`] is not finite or `<= 0`.
  InvalidDose { dose_mg: f64 },
  /// No `(N, V)` pair satisfies the selection criteria for this dose.
  /// Surfaced explicitly - never silently replaced by an unmeasurable volume.
  NoSafePlan { dose_mg: f64 },
}

impl fmt::Display for PlanError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidWeight { weight_kg } => {
        write!(
          f,
          "invalid weight {weight_kg} kg: must be a finite value greater than 0"
        )
      }
      Self::InvalidDose { dose_mg } => {
        write!(
          f,
          "invalid dose {dose_mg} mg: must be a finite value greater than 0"
        )
      }
      Self::NoSafePlan { dose_mg } => {
        write!(f, "no safe measurable plan found for a {dose_mg} mg dose")
      }
    }
  }
}

impl std::error::Error for PlanError {}

/// Builds the full Day 1 + Days 2-5 regimen for a body weight in kg.
///
/// Validates the weight (domain-level, independent of UI, §4.5): non-numeric,
/// non-finite, or `<= 0` weights are rejected. There is **no upper bound** on
/// weight (confirmed).
///
/// # Errors
///
/// Returns [`PlanError::InvalidWeight`] for `weight_kg <= 0`, NaN, or
/// infinite values; [`PlanError::NoSafePlan`] if no measurable plan exists
/// for the derived doses.
///
/// # Examples
///
/// ```
/// use favi_child::domain::{plan_for_weight, PlanError};
///
/// let plan = plan_for_weight(4.6).unwrap();
/// assert_eq!(plan.day1.tablets, 1.0);
/// assert_eq!(plan.day1.diluent_ml, 5.0);
/// assert_eq!(plan.day1.draw_ml, 4.0);
///
/// assert_eq!(plan_for_weight(0.0), Err(PlanError::InvalidWeight { weight_kg: 0.0 }));
/// ```
pub fn plan_for_weight(weight_kg: f64) -> Result<RegimenPlan, PlanError> {
  if !weight_kg.is_finite() || weight_kg <= 0.0 {
    return Err(PlanError::InvalidWeight { weight_kg });
  }
  let day1 = plan_for_dose(per_administration_mg(weight_kg, RegimenDay::Day1))?;
  let days2_5 = plan_for_dose(per_administration_mg(weight_kg, RegimenDay::Days2To5))?;
  Ok(RegimenPlan { day1, days2_5 })
}

/// Finds the best mixing plan `(N, V, D)` for a single per-dose amount.
///
/// Searches `N` over 0.5-tablet increments up to the guaranteed-valid
/// boundary and `V` over the two-size diluent set `{5, 10}` mL, then ranks
/// the valid candidates: **5 mL is tried first** (10 mL is only used when no
/// valid plan fits in 5 mL), then economy (smallest `N`), roundness (whole-mL
/// draw over half-mL), and finally smallest rounding delta (§4.3).
///
/// # Errors
///
/// Returns [`PlanError::InvalidDose`] for `dose_mg <= 0`, NaN, or infinite
/// values; [`PlanError::NoSafePlan`] when no candidate satisfies the
/// selection criteria (e.g. extremely low doses where even `D < 1.0 mL`).
///
/// # Examples
///
/// ```
/// use favi_child::domain::plan_for_dose;
///
/// let plan = plan_for_dose(161.0).unwrap();
/// assert_eq!((plan.tablets, plan.diluent_ml, plan.draw_ml), (1.0, 5.0, 4.0));
/// ```
pub fn plan_for_dose(dose_mg: f64) -> Result<MixingPlan, PlanError> {
  if !dose_mg.is_finite() || dose_mg <= 0.0 {
    return Err(PlanError::InvalidDose { dose_mg });
  }

  // Smallest 0.5-multiple >= dose/200: at V = 5 mL the draw volume there
  // is at most 5 mL, so at least one valid candidate always exists.
  let max_tablets = max_tablets_for(dose_mg);

  let mut candidates: Vec<MixingPlan> = Vec::new();
  let mut tablets = TABLET_FRACTION_STEP;
  while tablets <= max_tablets {
    for &diluent_ml in &DILUENT_VOLUMES_ML {
      if !is_valid_candidate(dose_mg, tablets, diluent_ml) {
        continue; // rules 2-4: measurable draw, fits the mixture, within tolerance
      }
      let concentration_mg_per_ml = tablets * TABLET_STRENGTH_MG / diluent_ml;
      let exact_draw_ml = dose_mg / concentration_mg_per_ml;
      let draw_ml = round_to_step(exact_draw_ml, DRAW_VOLUME_GRADUATION_ML);
      let delivered_mg = draw_ml * concentration_mg_per_ml;
      candidates.push(MixingPlan {
        dose_mg,
        tablets,
        diluent_ml,
        draw_ml,
        concentration_mg_per_ml,
        delivered_mg,
        delta_mg: delivered_mg - dose_mg,
      });
    }
    tablets += TABLET_FRACTION_STEP;
  }

  candidates.sort_by(|a, b| {
    a.diluent_ml
      .total_cmp(&b.diluent_ml) // diluent preference: 5 mL first, 10 mL fallback
      .then_with(|| a.tablets.total_cmp(&b.tablets)) // economy: fewest tablets
      .then_with(|| whole_ml(b.draw_ml).cmp(&whole_ml(a.draw_ml))) // roundness: whole-mL over half-mL
      .then_with(|| a.delta_mg.abs().total_cmp(&b.delta_mg.abs())) // smallest dosing delta
  });

  candidates
    .into_iter()
    .next()
    .ok_or(PlanError::NoSafePlan { dose_mg })
}

/// True when `(tablets, diluent_ml)` yields a draw volume meeting the hard
/// selection rules: measurable (`D >= 1.0 mL`, 0.5-mL multiple by rounding),
/// fits the mixture (`D <= V`), and within the rounding-error tolerance.
fn is_valid_candidate(dose_mg: f64, tablets: f64, diluent_ml: f64) -> bool {
  let concentration_mg_per_ml = tablets * TABLET_STRENGTH_MG / diluent_ml;
  let exact_draw_ml = dose_mg / concentration_mg_per_ml;
  let draw_ml = round_to_step(exact_draw_ml, DRAW_VOLUME_GRADUATION_ML);
  draw_ml >= MIN_DRAW_VOLUME_ML
    && draw_ml <= diluent_ml
    && (exact_draw_ml - draw_ml).abs() <= ROUNDING_TOLERANCE_ML
}

/// Smallest 0.5-multiple of tablets `>= dose / 200` - the search boundary
/// that guarantees at least one valid candidate exists for `dose_mg >= 20`.
fn max_tablets_for(dose_mg: f64) -> f64 {
  ceil_to_step(dose_mg / TABLET_STRENGTH_MG, TABLET_FRACTION_STEP)
}

/// Rounds a value to the nearest multiple of `step` (half away from zero).
fn round_to_step(value: f64, step: f64) -> f64 {
  (value / step).round() * step
}

/// Ceils a value to the nearest multiple of `step`.
fn ceil_to_step(value: f64, step: f64) -> f64 {
  (value / step).ceil() * step
}

/// True when a draw volume (multiple of 0.5 mL by construction) is whole-mL.
fn whole_ml(draw_ml: f64) -> bool {
  draw_ml.fract() == 0.0
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The confirmed worked example (AGENTS.md §4.3, §7 regression test):
  /// weight 4.6 kg → Day 1: 1 tablet / 5 mL / 4.0 mL draw.
  #[test]
  fn worked_example_weight_4_6_day1_matches_spec() {
    let plan = plan_for_weight(4.6).expect("4.6 kg must produce a plan");
    assert_eq!(plan.day1.dose_mg, 161.0);
    assert_eq!(plan.day1.tablets, 1.0);
    assert_eq!(plan.day1.diluent_ml, 5.0);
    assert_eq!(plan.day1.draw_ml, 4.0);
    assert_eq!(plan.day1.concentration_mg_per_ml, 40.0);
    assert_eq!(plan.day1.delivered_mg, 160.0);
    assert_eq!(plan.day1.delta_mg, -1.0);
  }

  /// Days 2-5 for 4.6 kg: dose 69 mg. 5 mL is tried first and works
  /// (N = 0.5, D = 3.5), so 10 mL is never used.
  #[test]
  fn worked_example_weight_4_6_days2_5_prefers_5_ml_diluent() {
    let plan = plan_for_weight(4.6).expect("4.6 kg must produce a plan");
    assert_eq!(plan.days2_5.dose_mg, 69.0);
    assert_eq!(plan.days2_5.tablets, 0.5);
    assert_eq!(plan.days2_5.diluent_ml, 5.0);
    assert_eq!(plan.days2_5.draw_ml, 3.5);
  }

  /// Rule 5: the Days 2-5 plan is computed once and must be deterministic -
  /// calling the calculator twice yields identical plans.
  #[test]
  fn days2_5_plan_is_deterministic() {
    for weight in (10..=300).map(|t| t as f64 / 10.0) {
      let a = plan_for_weight(weight).expect("plan must exist");
      let b = plan_for_weight(weight).expect("plan must exist");
      assert_eq!(a.days2_5, b.days2_5);
    }
  }

  /// §4.5: weights that are non-numeric / not finite / <= 0 are rejected.
  #[test]
  fn invalid_weights_are_rejected() {
    for weight in [-1.0, 0.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
      let err = plan_for_weight(weight).expect_err("must be rejected");
      assert!(matches!(err, PlanError::InvalidWeight { .. }));
    }
  }

  /// §4.5: extremely low weights must surface an explicit "no safe plan"
  /// state instead of a silently-invalid draw volume.
  #[test]
  fn extremely_low_weight_returns_no_safe_plan() {
    let err = plan_for_weight(0.05).expect_err("0.05 kg must have no safe plan");
    assert!(matches!(err, PlanError::NoSafePlan { dose_mg } if dose_mg < 2.0));
  }

  /// plan_for_dose rejects non-positive doses directly.
  #[test]
  fn invalid_dose_is_rejected() {
    for dose in [-5.0, 0.0, f64::NAN] {
      let err = plan_for_dose(dose).expect_err("must be rejected");
      assert!(matches!(err, PlanError::InvalidDose { .. }));
    }
  }

  /// §7 property-style check: for a spread of weights the algorithm always
  /// terminates with a valid plan or an explicit rejection - never a panic,
  /// never a silently-invalid draw volume.
  #[test]
  fn property_check_all_weights_terminate_with_valid_plan_or_rejection() {
    for weight in (1..=1500).map(|t| t as f64 / 10.0) {
      match plan_for_weight(weight) {
        Ok(plan) => {
          assert_invariants(&plan.day1, weight, "day1");
          assert_invariants(&plan.days2_5, weight, "days2_5");
        }
        Err(PlanError::NoSafePlan { .. }) => {
          // Days 2-5 needs dose >= 7.5 mg for any measurable plan
          // with V <= 10 mL, i.e. weight >= 0.5 kg; below that
          // NoSafePlan is correct.
          assert!(weight < 0.5, "unexpected NoSafePlan for {weight} kg");
        }
        Err(other) => panic!("unexpected error for {weight} kg: {other}"),
      }
    }
  }

  /// Table-driven dose checks (AGENTS.md §7): mechanical weights with
  /// invariant assertions only - clinical test values await เวช's sign-off.
  #[test]
  fn table_driven_doses_keep_invariants() {
    for weight in [1.0, 2.5, 5.0, 10.0, 20.0, 35.0] {
      let plan = plan_for_weight(weight).expect("plan must exist");
      assert_invariants(&plan.day1, weight, "day1");
      assert_invariants(&plan.days2_5, weight, "days2_5");
    }
  }

  /// Invariants that must hold for every produced plan (§4.3 rules 1-2, §4.4).
  fn assert_invariants(plan: &MixingPlan, weight: f64, label: &str) {
    let ctx = format!("{weight} kg {label}");
    // measurability: draw volume is a multiple of 0.5 mL
    assert!(
      (plan.draw_ml * 2.0).fract().abs() < 1e-9,
      "{ctx}: draw {:.2} mL not a 0.5-multiple",
      plan.draw_ml
    );
    // correctness: draw volume fits the mixture and is not too small
    assert!(
      plan.draw_ml >= MIN_DRAW_VOLUME_ML && plan.draw_ml <= plan.diluent_ml,
      "{ctx}: draw {:.2} mL outside [{MIN_DRAW_VOLUME_ML}, {:.1}] mL",
      plan.draw_ml,
      plan.diluent_ml
    );
    // tablet count is a multiple of the 0.5-tablet step
    assert!(
      (plan.tablets / TABLET_FRACTION_STEP).fract().abs() < 1e-9,
      "{ctx}: tablets {:.1} not a 0.5-multiple",
      plan.tablets
    );
    // diluent comes from the confirmed candidate set
    assert!(
      DILUENT_VOLUMES_ML.contains(&plan.diluent_ml),
      "{ctx}: diluent {:.1} mL not in confirmed set",
      plan.diluent_ml
    );
    // QC consistency: delivered = draw * concentration, delta = delivered - dose
    assert!((plan.delivered_mg - plan.draw_ml * plan.concentration_mg_per_ml).abs() < 1e-6);
    assert!((plan.delta_mg - (plan.delivered_mg - plan.dose_mg)).abs() < 1e-6);
    // rounding delta never exceeds the tolerance amplified to mg
    assert!(plan.delta_mg.abs() <= ROUNDING_TOLERANCE_ML * plan.concentration_mg_per_ml + 1e-6);
  }

  /// Diluent preference + economy: the selected plan must use the smallest
  /// diluent size that has any valid candidate, and within that size the
  /// smallest tablet count. Re-derives the search to confirm no better
  /// candidate was skipped.
  #[test]
  fn selection_prefers_5_ml_then_smallest_tablet_count() {
    for weight in (5..=500).map(|t| t as f64 / 10.0) {
      let plan = plan_for_weight(weight).expect("plan must exist");
      for day_plan in [&plan.day1, &plan.days2_5] {
        // no valid candidate in a smaller diluent size
        for &diluent_ml in &DILUENT_VOLUMES_ML {
          if diluent_ml >= day_plan.diluent_ml {
            break;
          }
          let mut tablets = TABLET_FRACTION_STEP;
          while tablets <= max_tablets_for(day_plan.dose_mg) {
            assert!(
              !is_valid_candidate(day_plan.dose_mg, tablets, diluent_ml),
              "a valid candidate exists with {diluent_ml:.1} mL for dose {:.1} mg \
                             but the plan uses {:.1} mL",
              day_plan.dose_mg,
              day_plan.diluent_ml
            );
            tablets += TABLET_FRACTION_STEP;
          }
        }
        // no valid candidate with the same diluent size and fewer tablets
        let mut tablets = TABLET_FRACTION_STEP;
        while tablets < day_plan.tablets {
          assert!(
            !is_valid_candidate(day_plan.dose_mg, tablets, day_plan.diluent_ml),
            "a smaller tablet count than {:.1} had a valid candidate for dose {:.1} mg",
            day_plan.tablets,
            day_plan.dose_mg
          );
          tablets += TABLET_FRACTION_STEP;
        }
      }
    }
  }
}
