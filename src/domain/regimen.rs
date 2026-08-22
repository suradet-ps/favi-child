//! Day-by-day per-dose formulas (AGENTS.md §4.2).
//!
//! ```text
//! day1_dose_per_administration_mg   = 35 * weight_kg
//! day2_5_dose_per_administration_mg = 15 * weight_kg
//! ```
//!
//! The only calculator input is `weight_kg`; there is no age or any other
//! parameter per the current spec.

/// Which part of the regimen a dose belongs to (§4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegimenDay {
    /// Day 1 - 70 mg/kg/day total, split into 2 doses: 35 mg/kg per dose.
    Day1,
    /// Days 2-5 - 30 mg/kg/day total, split into 2 doses: 15 mg/kg per dose.
    Days2To5,
}

impl RegimenDay {
    /// Per-dose mg per kg of body weight for this part of the regimen.
    pub const fn per_dose_factor(self) -> f64 {
        match self {
            Self::Day1 => 35.0,
            Self::Days2To5 => 15.0,
        }
    }
}

/// Per-administration dose in mg (AM or PM) for a given weight and day.
///
/// # Examples
///
/// ```
/// use favi_child::domain::{per_administration_mg, RegimenDay};
///
/// assert_eq!(per_administration_mg(4.6, RegimenDay::Day1), 161.0);
/// assert_eq!(per_administration_mg(4.6, RegimenDay::Days2To5), 69.0);
/// ```
pub fn per_administration_mg(weight_kg: f64, day: RegimenDay) -> f64 {
    day.per_dose_factor() * weight_kg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day1_factor_is_35_mg_per_kg() {
        assert_eq!(per_administration_mg(1.0, RegimenDay::Day1), 35.0);
        assert_eq!(per_administration_mg(4.6, RegimenDay::Day1), 161.0);
    }

    #[test]
    fn days2_5_factor_is_15_mg_per_kg() {
        assert_eq!(per_administration_mg(1.0, RegimenDay::Days2To5), 15.0);
        assert_eq!(per_administration_mg(4.6, RegimenDay::Days2To5), 69.0);
    }

    #[test]
    fn zero_weight_yields_zero_dose() {
        assert_eq!(per_administration_mg(0.0, RegimenDay::Day1), 0.0);
        assert_eq!(per_administration_mg(0.0, RegimenDay::Days2To5), 0.0);
    }
}
