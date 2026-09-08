# Changelog

All notable changes to this project are documented here, following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Project scaffold: Cargo.toml (Leptos 0.8 CSR), trunk `index.html`, DESIGN.md
  design tokens in `styles.css`.
- `domain` module (pure Rust, zero Leptos deps):
  - `constants` - protocol constants (tablet strength, diluent set, syringe
    graduation, provisional rounding tolerance).
  - `regimen` - per-day dose formulas (35 / 15 mg per kg per dose).
  - `plan` - reconstitution search over `(N, V)` pairs with correctness,
    measurability, roundness, and economy selection; explicit
    `NoSafePlan` rejection path.
- Unit tests: confirmed worked example (4.6 kg → 1 tab / 5 mL / 4.0 mL),
  invalid-weight rejection, no-safe-plan boundary, Days 2-5 consistency,
  property-style termination/invariant checks over a weight spread.
- Leptos components: `App` (root signal wiring + derived plans),
  `WeightInput`, `DayPlanCard` (renders §4.4 output incl. QC delta and
  "รับประทานเช้า-เย็น").
- OSS files: MIT license, README, this changelog, `.gitignore`.

### Changed

- Supporting docs moved to `docs/`: `AGENTS-RUST.md`, `DESIGN.md`, and this
  changelog. Path references updated in `AGENTS.md`, `README.md`, and code
  comments (`docs/DESIGN.md`, `docs/AGENTS-RUST.md`).
- Diluent set reduced to two sizes `{5, 10}` mL - 5 mL is tried first, 10 mL
  is the fallback when no valid plan fits in 5 mL (user-confirmed protocol
  change; pending เวช sign-off). Search boundary and `NoSafePlan` threshold
  (weight < 0.5 kg) updated accordingly.
- Syringe graduation refined from 0.5 mL to **0.2 mL** steps (user-confirmed;
  supersedes the earlier whole/half-mL graduation). Draw volumes now round to
  the nearest 0.2 mL (e.g. 2.2, 2.4, 2.6 mL); provisional rounding tolerance
  adjusted to 0.1 mL (half of the new step, pending เวช sign-off). Days 2-5
  `NoSafePlan` threshold moves to weight < 0.6 kg (a 9 mg dose is the
  smallest measurable, via 0.5 tablet / 10 mL / 1.0 mL draw).

### Pending clinical confirmation

- Rounding-error tolerance for the whole-mL tie-break (provisional 0.1 mL).
- Minimum-weight plausibility bound (not enforced yet).