# FaviChild

```
███████╗ █████╗ ██╗   ██╗██╗
██╔════╝██╔══██╗██║   ██║██║
█████╗  ███████║██║   ██║██║
██╔══╝  ██╔══██║╚██╗ ██╔╝██║
██║     ██║  ██║ ╚████╔╝ ██║
╚═╝╚═╝  ╚═╝  ╚═══╝╚═╝
```

---

## ◆ PULSE

A child's dose of Favipiravir cannot be rounded into a hope. FaviChild
turns one input - body weight in kilograms - into a day-by-day plan
for preparing pediatric liquid suspension from 200 mg tablets: how
many tablets to crush, how much diluent to use, and how much to draw
per dose, AM and PM, for Day 1 and Days 2-5. Every rounding is shown,
never hidden - the delivered mg and its delta against the theoretical
dose sit beside the plan for the pharmacist's QC. When no safe,
measurable plan exists, the page says so plainly.

| Weight in ▣ | Day plans ▣ | Honest rounding ▣ | No-solution state ▣ |
|---|---|---|---|

*The plan loop - derive, measure, reveal, refuse - is sealed.*

> Built with Leptos 0.8 + trunk; the clinical math lives in pure Rust
> (`src/domain/`), independently unit-tested, untouched by the UI.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One target, one tool, one command.

```
⟫ rustup target add wasm32-unknown-unknown
⟫ cargo install trunk
⟫ trunk serve
```

Open [http://127.0.0.1:8080](http://127.0.0.1:8080).

The release artifact: `⟫ trunk build` - output in `dist/`.

<details>
<summary>Prerequisites</summary>

- Rust with the `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) - installed above

</details>

---

## ◆ ANATOMY

One signal, one domain, a boundary the math never crosses.

- **Derives** - weight in, two regimens out: Day 1 at
  35 mg/kg/dose, Days 2-5 at 15 mg/kg/dose, with Days 2-5 reusing a
  single mixing plan computed once.
- **Plans** - reconstitution in honest steps: tablet count `N` in
  0.5-tablet steps, diluent volume `V` from two sizes only (5 mL
  tried first, 10 mL fallback), and draw volume `D` in 0.5 mL
  multiples.
- **Reveals** - the actual delivered mg per dose and its delta
  versus the theoretical dose - the rounding is on the page, where
  the pharmacist can judge it, not buried in the algorithm.
- **Refuses** - when no safe measurable plan exists, the state says
  "no safe measurable plan found" instead of quietly printing an
  invalid number.
- **Separates** - `src/domain/` is pure Rust with zero Leptos
  dependencies: all dosing logic lives there, unit-testable without a
  browser; `src/components/` renders and never computes.

---

## ◆ RITUALS

**The core ceremony** - the preparation plan:

1. Enter the child's weight. One number in, one plan out.
2. Read the Day 1 plan: tablets, diluent, draw volume, AM and PM.
3. Check the QC line: delivered mg and the delta against the
   theoretical dose - the rounding in plain sight.
4. When no measurable plan exists, trust the refusal - the page
   says so before the syringe is touched.

**The ceremony of the visible rounding** - every plan is a set of
measurable steps: 0.5-tablet steps, 5 or 10 mL diluent, 0.5 mL draws.
What cannot be measured is not silently approximated; it is refused.

**The ceremony of the pending review** - the protocol constants wait
for the pharmacist's (เวช) sign-off, and the disclaimer says so on
every use. The tool is for evaluation until the review lands; the
review is the door, and the door is labeled.

---

## ◆ ECHOES

**Where this artifact is heading**

```
derive  ▸ Day 1 + Days 2-5 regimens from weight ────────────────────── ▸ sealed
measure ▸ 0.5-step tablets, 5/10 mL diluent, 0.5 mL draws ──────────── ▸ sealed
reveal  ▸ delivered mg + delta QC line ──────────────────────────────── ▸ sealed
refuse  ▸ explicit no-safe-plan state ───────────────────────────────── ▸ sealed
```

**Raising the artifact** - the clinical protocol lives in
`AGENTS.md` §5-§8; the design tokens in `docs/DESIGN.md`; the CI
checklist in `docs/AGENTS-RUST.md` §11. Gates: `cargo fmt --check`,
`cargo check`, `cargo clippy -- -D warnings`, `cargo test`, and
`cargo doc --no-deps` free of warnings. Open an issue first to
discuss a change.

**Status** - CI gates every push. [Watch the gates](.github/workflows).

> Clinical disclaimer: the dosing constants encode a specific protocol
> pending pharmacist review before any clinical use. This tool is for
> evaluation and development purposes only.

---

```
  ─────────────────────────────────────────
   A dose that cannot be measured
   is a dose that must not be given.
  ─────────────────────────────────────────
```

MIT - see [LICENSE](LICENSE).