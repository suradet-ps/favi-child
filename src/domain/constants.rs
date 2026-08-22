//! Medication and protocol constants (AGENTS.md §4.1, §4.3, §8).
//!
//! ⚠️ These values encode a specific clinical dosing protocol. They must be
//! reviewed and confirmed by the pharmacist before release. Any change to
//! these numbers is a clinical decision, not an engineering one.

/// Strength of one Favipiravir tablet, in mg (AGENTS.md §4.1).
pub const TABLET_STRENGTH_MG: f64 = 200.0;

/// Smallest physically splittable tablet unit - ½ tablet = 100 mg (§4.1).
/// Tablet counts are always multiples of this step.
pub const TABLET_FRACTION_STEP: f64 = 0.5;

/// Doses per day: AM and PM (§4.1).
pub const DOSES_PER_DAY: u32 = 2;

/// Confirmed candidate set of diluent (water) volumes in mL (§8, decision 1).
///
/// Revised (user-confirmed): only two sizes, **5 mL tried first**, falling
/// back to 10 mL when no valid plan fits in 5 mL. Pending เวช sign-off.
pub const DILUENT_VOLUMES_ML: [f64; 2] = [5.0, 10.0];

/// Syringe graduation: draw volumes are multiples of 0.5 mL (§8, decision 2).
pub const DRAW_VOLUME_GRADUATION_ML: f64 = 0.5;

/// Draw volumes below this are rejected: too small to measure precisely (§4.3 rule 2).
pub const MIN_DRAW_VOLUME_ML: f64 = 1.0;

/// Provisional rounding-error tolerance (mL) for the §4.3 rule 3 tie-break.
///
/// ⚠️ **Pending pharmacist confirmation** (§8, open item 2). The provisional
/// value is 0.25 mL - the maximum error possible when rounding a draw volume
/// to the nearest 0.5 mL - so no candidate is ever filtered out by it and the
/// whole-mL preference acts purely as a tie-break, as §4.3 describes.
pub const ROUNDING_TOLERANCE_ML: f64 = 0.25;

/// Minimum weight plausibility bound (kg).
///
/// ⚠️ **Pending pharmacist confirmation** (§8, open item 1). The spec's
/// confirmed rules only reject `weight <= 0` / non-numeric input (§4.5); the
/// low-end plausibility cutoff (to catch typos) is intentionally **not**
/// enforced until เวช confirms it. Below ~0.5 kg (Days 2-5 dose below
/// 7.5 mg) the search naturally returns
/// [`PlanError::NoSafePlan`](crate::domain::plan::PlanError::NoSafePlan).
pub const MIN_PLAUSIBLE_WEIGHT_KG: Option<f64> = None;
