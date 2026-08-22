# AGENTS.md — FaviChild

## 1. Project Overview

**FaviChild** is a client-side web application for calculating pediatric
liquid-suspension dosing of **Favipiravir** (200 mg tablets, no liquid
formulation available). It converts a child's body weight (kg) into a
day-by-day administration plan: how many tablets to crush/dissolve, how much
diluent to use, and how much suspension volume to draw and administer per
dose (AM/PM), for Day 1 and Days 2–5 of the regimen.

This document defines **architecture and domain rules only**. Visual design
(color tokens, spacing, typography, component styling) is the single
responsibility of `docs/DESIGN.md` — do not duplicate or restate design tokens
here; reference `docs/DESIGN.md` wherever a UI decision is needed.

Rust coding conventions (idioms, error handling, module layout style) are
covered by `docs/AGENTS-RUST.md`, not restated here.

## 2. Scope

**In scope:**
- Weight-based dose calculation for Day 1 and Days 2–5.
- Tablet/diluent/draw-volume reconstitution planning (the "how to actually
  prepare and measure the dose" logic — this is the core value of the app).
- Presenting the plan as a clear, printable/shareable instruction sheet.

**Out of scope (v1):**
- Any medication other than Favipiravir.
- Any tablet strength other than 200 mg.
- Weight-tracking, patient records, or persistence across sessions.
- Backend/server — this is a pure client-side (CSR/WASM) tool
  integration, no database.

## 3. Tech Stack

- **Leptos 0.8**, CSR (client-side rendering) — compiled to WASM, no SSR,
  no server function boundary. Static hosting target (e.g., a single
  `index.html` + wasm bundle).
- No Tauri wrapper in v1 — this is a **browser web app**, not a desktop app.
  (If a desktop wrapper is added later, that is a separate ADR and a new
  section here — do not assume it.)
- Build tooling: `trunk` (standard for Leptos CSR projects), unless
  `docs/DESIGN.md` or a future decision specifies otherwise.
- No external state library — Leptos signals are sufficient given the small,
  single-screen state shape (see §6).

## 4. Domain Model & Clinical Rules

> ⚠️ These constants and rules encode a specific clinical dosing protocol.
> They must be reviewed and confirmed by the pharmacist before release.
> Any change to these numbers is a clinical decision, not an engineering one.

### 4.1 Medication constants

| Constant | Value |
|---|---|
| Tablet strength | 200 mg / tablet |
| Smallest physically splittable unit | ½ tablet = 100 mg |
| Doses per day | 2 (AM and PM) |

### 4.2 Regimen table

| Day | Total daily dose | Per-dose (AM or PM) |
|---|---|---|
| Day 1 | 70 mg/kg/day | 35 mg/kg |
| Day 2–5 | 30 mg/kg/day | 15 mg/kg |

Formulas:
```
day1_dose_per_administration_mg   = 35 * weight_kg
day2_5_dose_per_administration_mg = 15 * weight_kg
```

Input: `weight_kg: f64` (only input the calculator requires — no age, no
other parameters per current spec).

### 4.3 Reconstitution / administration-plan algorithm

This is the core domain logic and the reason the app exists: a required
dose in mg (§4.2) rarely corresponds to a physically measurable fraction of
a 200 mg tablet, so the plan is **not** "give X tablets" but "dissolve N
tablets in V mL of water, then draw and administer D mL."

**Given:** a required per-dose amount `dose_mg`.

**Find:** a tuple `(N, V, D)` where:
- `N` = number of tablets to crush/dissolve, in 0.5-tablet increments
  (i.e., multiples of 100 mg): `N ∈ {0.5, 1, 1.5, 2, ...}`
- `V` = diluent (water) volume in mL, chosen from the confirmed candidate
  set `{5, 10, 15, 20, 25, 30}` mL
- `D` = draw volume in mL, administered via oral syringe

**Relationship:**
```
concentration_mg_per_mL = (N * 200) / V
D = dose_mg / concentration_mg_per_mL
```

**Selection criteria** (search over candidate `N`, `V` pairs to pick the
best plan):
1. **Correctness:** `D <= V` (cannot draw more than what was mixed).
2. **Measurability:** `D` must be a **multiple of 0.5 mL** (i.e., only
   whole-mL or half-mL draw volumes are allowed — confirmed syringe
   graduation). Round the mathematically exact draw volume to the nearest
   0.5 mL. Also avoid `D < 1.0 mL` (too small to measure precisely).
3. **Roundness:** since `D` is already constrained to 0.5 mL increments by
   rule 2, this criterion is about tie-breaking: when multiple `(N, V)`
   pairs round to acceptably close 0.5 mL draw volumes, prefer whole-mL
   values over half-mL values (e.g., prefer `D = 4.0` over `D = 4.5`) when
   both are within acceptable dosing error tolerance (tolerance value:
   **to be confirmed**, see §8).
4. **Economy:** prefer the smallest `N` (fewest tablets consumed/wasted)
   among plans that satisfy 1–3.
5. **Consistency across Day 2–5 (mandatory, confirmed):** Days 2–5 use
   **exactly one** `(N, V)` mixing plan, computed once, since the per-dose
   mg is constant across those 4 days. The caregiver mixes once and reuses
   the same concentration for all four days — the algorithm must not
   produce a different plan for, e.g., Day 3 vs. Day 4.

**Worked example (validates against the spec's given example):**
```
weight_kg = 4.6   (illustrative)
day1_dose_per_administration_mg = 35 * 4.6 = 161 mg

Candidate: N = 1 tablet (200 mg), V = 5 mL
  concentration = 200 / 5 = 40 mg/mL
  D = 161 / 40 = 4.025 mL → rounds to nearest 0.5 mL = 4.0 mL  ✅ matches example
```
This confirms the algorithm shape: **do not** round the tablet count to
match the dose exactly; instead fix a workable `(N, V)` and let the *draw
volume* absorb the precision, since volume is what can be measured finely.

### 4.4 Output per day

For each of Day 1 and Days 2–5, the plan must state:
- Tablet count to crush/dissolve (`N`)
- Diluent volume (`V`, mL of water)
- Draw volume per administration (`D`, mL)
- Explicit instruction: "รับประทานเช้า-เย็น" (administer AM and PM)
- Resulting actual mg delivered per dose (`D * concentration`), for
  pharmacist QC — this may differ slightly from the theoretical `dose_mg`
  due to rounding, and that delta should be visible, not hidden.

### 4.5 Safety guards (domain-level validation, independent of UI)

- Reject or flag `weight_kg <= 0` or non-numeric input.
- **No upper bound** on weight (confirmed) — do not cap or reject large
  weights.
- Minimum-weight plausibility bound: still **to be confirmed with เวช**
  (see §8) — needed only to catch obviously-invalid low input (e.g.,
  typos), not as a clinical dosing ceiling.
- If no `(N, V)` pair satisfies the selection criteria in §4.3 for a given
  weight (e.g., extremely low weight), the calculator must surface this as
  an explicit "no safe measurable plan found" state — never silently fall
  back to an unmeasurable draw volume.

## 5. Architecture

Given the small surface area (single calculator, no persistence, no
network calls), a full hexagonal/ports-style split is not warranted for
v1. Instead:

```
favi-child/
├── src/
│   ├── main.rs              # Leptos app entry / mount
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── constants.rs     # §4.1 constants
│   │   ├── regimen.rs       # §4.2 dose-per-day formulas
│   │   └── plan.rs          # §4.3 reconstitution search algorithm
│   ├── components/
│   │   ├── weight_input.rs
│   │   ├── day_plan_card.rs # renders one day's plan (§4.4)
│   │   └── app.rs           # top-level component, wires signals
│   └── ...
├── AGENTS.md                # this file
└── docs/                    # supporting specs (referenced, not restated)
    ├── AGENTS-RUST.md       # Rust conventions
    └── DESIGN.md            # visual design tokens
```

**Key principle:** the `domain/` module is pure Rust, has **zero Leptos
dependencies**, and is independently unit-testable. All clinical logic
(§4) lives here. `components/` only calls into `domain/` and renders
results — it must not contain dosing math.

## 6. State Management

Single source of truth: `weight_kg: RwSignal<Option<f64>>` at the app root.
Day-plan results are derived (`Memo`/`Signal::derive`) from this signal by
calling into `domain::plan`, not stored as separate mutable signals — this
guarantees the displayed plan can never drift out of sync with the entered
weight.

## 7. Testing Strategy

Because incorrect output here has direct patient-safety consequences,
`domain/` requires unit tests **before** UI work begins:

- Table-driven tests covering a spread of realistic pediatric weights
  (values to be supplied by เวช — do not invent clinical test weights
  without pharmacist sign-off).
- Explicit regression test for the worked example in §4.3 (weight ≈4.6 kg
  → 1 tablet / 5 mL / 4.0 mL draw).
- Boundary tests for the "no safe plan found" path (§4.5).
- Property-style check: for all weights in the accepted range, the
  algorithm must always terminate with either a valid plan or an explicit
  rejection — never a panic, never a silently-invalid `D`.

## 8. Decisions Log

1. **Diluent volume set:** `{5, 10, 15, 20, 25, 30}` mL — confirmed.
2. **Syringe graduation:** draw volume `D` must be a multiple of **0.5 mL**
   (whole or half mL only) — confirmed. See §4.3 rule 2.
3. **Weight bounds:** **no upper bound** on weight — confirmed. See §4.5.
4. **Day 2–5 mixing plan:** identical `(N, V)` plan reused across all four
   days, computed once — confirmed. See §4.3 rule 5.

Still open (need เวช's decision before implementation):

1. **Minimum-weight plausibility bound** — exact low-end cutoff for input
   validation (to catch typos, not a clinical ceiling). See §4.5.
2. **Dosing error tolerance** for the tie-break rule in §4.3 rule 3 (when
   choosing between a whole-mL vs. half-mL draw volume that are both
   within acceptable range of the exact dose).
3. **Output format** beyond on-screen display — printable sheet? Thai-
   language instruction text baked into the UI, or component-only with
   copy handled via `docs/DESIGN.md`

These remain flagged rather than assumed because they are clinical/protocol
decisions, not engineering defaults.
