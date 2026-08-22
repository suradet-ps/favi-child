# FaviChild

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A client-side (CSR/WASM) web app that converts a child's body weight (kg)
into a day-by-day plan for preparing pediatric liquid-suspension Favipiravir
(200 mg tablets): how many tablets to crush/dissolve, how much diluent
(water) to use, and how much suspension to draw and administer per dose
(AM/PM), for Day 1 and Days 2-5 of the regimen.

Built with **Leptos 0.8** and **trunk**. No backend, no database, no
persistence - a single static page + wasm bundle.

> **Clinical disclaimer**: the dosing protocol constants encode a specific
> clinical protocol and are **pending pharmacist (เวช) review before any
> clinical use**. Some values are still explicitly open (see
> [Open clinical decisions](#open-clinical-decisions)). This tool is for
> evaluation and development purposes only.

## Features

- Weight-only input; derives Day 1 (35 mg/kg/dose) and Days 2-5 (15 mg/kg/dose) plans.
- Reconstitution planning: tablet count `N` (0.5-tablet steps), diluent volume `V`
  (two sizes only: **5 mL tried first**, 10 mL fallback), and draw volume `D`
  (multiples of 0.5 mL).
- Days 2-5 reuse a single mixing plan - computed once (§4.3 rule 5).
- Shows the actual delivered mg per dose and its delta vs. the theoretical dose
  for pharmacist QC - rounding is visible, never hidden.
- Explicit "no safe measurable plan found" state instead of silently-invalid output.

## Getting started

```bash
# build + serve with hot reload (requires wasm32-unknown-unknown target)
trunk serve

# production build (outputs to dist/)
trunk build
```

Then open http://127.0.0.1:8080 (trunk's default dev server port).

## Development

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps 2>&1 | grep warning
```

The full CI checklist lives in `docs/AGENTS-RUST.md` §11.

## Architecture

`src/domain/` is pure Rust with zero Leptos dependencies - all clinical logic
lives here and is independently unit-testable. `src/components/` only calls
into `domain/` and renders results; it never contains dosing math. State is a
single `RwSignal<Option<f64>>` at the app root, with day plans derived from it
(AGENTS.md §5-§6). Visual design tokens: `docs/DESIGN.md`.

## Open clinical decisions

Per `AGENTS.md` §8, these still await เวช's decision and are **not assumed**:

1. Minimum-weight plausibility bound (low-end input cutoff).
2. Dosing error tolerance for the whole-mL/half-mL tie-break - implemented with
   a provisional `ROUNDING_TOLERANCE_ML = 0.25` mL (the maximum error possible
   at a 0.5 mL graduation), so it never filters out candidates.
3. Output format beyond on-screen display (printable sheet, etc.).

## License

MIT - see [LICENSE](LICENSE).