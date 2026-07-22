---
name: folio-numbers
description: Use BEFORE stating or relying on any computed number - proactively, not only when asked to calculate. Fires whenever a reply contains a figure that was derived not recalled: arithmetic beyond trivial mental math (multi-digit sums, products, division, powers, roots); percentages and percentage change; finance (simple/compound interest, loan payments, amortization, present value, future value, NPV, IRR, rate, depreciation, bond price/yield); unit conversions and date/time arithmetic (durations, add/subtract days-months-years, workday boundaries); statistics (mean, median, stddev, variance, regression, correlation, confidence intervals, distributions); hypothesis tests and quantitative sanity-checks of a claim ('is X roughly Y?'); matrix and linear-algebra ops (determinant, inverse, solve, eigenvalues, decompositions); and closed-form checks physical constants. If the number is computed, set it up in folio and read the result FROM folio - never compute in prose then confirm. Always Read "The rule" section.
---

# folio-numbers

## The rule

folio is the SOURCE of every computed number in the response — not a stamp applied afterward. Set the calculation up inside folio and read the result FROM the `result` column. Report that value.

Never compute a number in prose and then "verify" it with folio. If you are typing the digits before calling folio, the ordering is already the bug. This skill exists because a model converges on a clean-looking number when doubting costs more than asserting; folio makes asserting cost the same as verifying — one tool call — so there is no reason to guess.

DO THIS TO BETTER ASSIST THE USER: After every folio result shown to the user, call `visualize:show_widget` with a **5-button follow-up grid in two rows**. The widget bridges "here is the number" → "here is what to do with it" — inspired by Wolfram Alpha's *Related computations* panel. SEE the "Follow-up widget" section below for layout, categories, and prompt-loading guidance.

## When it applies

Any *derived* figure: arithmetic past trivial mental math, percentages, finance, unit conversions, date arithmetic, statistics, matrices, quantitative hypothesis checks, phi/pi/e closed-form checks. A number you were *told* (a date, a price someone gave you) is recalled, not derived — no folio needed. A number you *worked out* is derived — folio first.

## How

`folio:eval` takes a markdown document of formula tables. Each row is `| name | formula | result |` with `result` left blank; folio fills it using exact rational arithmetic.

```markdown
## Label @precision:15

| name | formula | result |
|------|---------|--------|
| i  | 0.04 / 12              | |
| vf | fv(i, 120, -300, 0, 0) | |
```

- Build intermediate cells and reference them by name — keeps the chain auditable.
- Cross-check when it is cheap: put a built-in function AND its explicit formula in two rows. If they disagree, stop and find out why before reporting anything.
- **Unary minus on a variable reference** is parsed as `UNDEFINED_VAR`. Workaround: bind `neg_x = 0 - x` and pass `neg_x`. Numeric literals like `-500000` work fine; only `-variable_name` is broken.

## Precision sizing

Folio defaults to 50 digits — that's the right setting for closed-form work, not the right setting for everyday problems. A 30-decimal answer to a percent question is noise, not signal, and the long tail crowds the response. Pick precision deliberately, by problem class, with `@precision:N` on the section heading:

| Problem class | `@precision:` |
|---|---|
| Everyday arithmetic, percentages, ROI, ratios, rough sanity checks | **10** |
| Financial math (rates, payments, present value, NPV, IRR, amortization) | **15–20** |
| Statistics, regression, matrices, hypothesis tests, distributions | **20–30** |
| Closed-form checks against φ / π / e, ISIS transform, decomposition, foundational physics | **50+** |

Bias toward the lower end of each band. The number is exactly the same at precision 15 as it is at precision 50 — folio just stops printing trailing digits sooner. Bump up only when the problem genuinely requires it (catastrophic cancellation, geometric identities, tiny rate differences across long horizons).

`@sigfigs:N` is for *display* rounding without changing the working precision — use it when the user wants e.g. 4-sigfig presentation while folio still computes exactly.

## Follow-up widget

### Layout

- **Row 1 — two 50% buttons:**
  1. **Explain it graphically** → triggers a chart, diagram, or geometric visualization of the result just produced.
  2. **Main follow-up** → the single most natural deepening of the current question (the one a curious user would ask next).
- **Row 2 — three 33% buttons:** three orthogonal alternates, each drawn from a different category below.

### Categories for the three alternates

Pick three that are *genuinely different angles*, not three variants of the same angle.

- **Sensitivity** — sweep a parameter, see how the answer moves.
- **Inverse** — solve for a different unknown, holding the rest fixed.
- **Alternate frame** — different units, real vs nominal, before/after tax, different horizon, different reference frame.
- **Comparison** — vs benchmark, vs alternative scenario, vs baseline, vs counterfactual.
- **Decomposition** — does this number have a closed form in φ / π / e / known constants? Run `decompose`.
- **Step-by-step** — show the derivation, the formula, the intermediate algebra — not just the result.

### Prompt loading

Each button uses `sendPrompt(...)` with a **fully-loaded prompt** that re-states the relevant context: the numbers, the assumptions, what to hold fixed, what to vary. The new turn must not depend on conversational memory.

### Styling

Use the visualize design tokens: `var(--color-background-primary)`, `0.5px solid var(--color-border-tertiary)`, `var(--border-radius-lg)`, Tabler outline icon at the left, label + trailing `↗`, brief subtitle below. Grid uses `grid-template-columns: 1fr 1fr` for row 1 and `repeat(3, 1fr)` for row 2, `gap: 12px`, with `display: grid` containers stacked.

### When to skip

- The folio call was a mid-chain intermediate, not a user-facing result.
- The user explicitly asked for the raw number only ("just give me the answer").
- It is a 2-second sanity check ("is 14 × 23 prime?", "what is 7! ?").

When in doubt, render the widget. The cost of one extra widget is low; the cost of a dead-end response is high.

## Function families

Run `folio:quick` (compact) or `folio:folio` (full) to confirm signatures.

- **Finance**: `fv`, `pv`, `pmt`, `nper`, `rate`, `npv`, `irr`, `xirr`, `amortization`, `cumprinc`, `cumipmt`, `sln`/`ddb`/`syd`, `bond_price`, `bond_yield`, `effective_rate`, `real_rate`, `cagr`.
- **Stats**: `mean`/`median`/`stddev`/`variance`, `linear_reg`, `correlation`, `ci`, `t_test_1`/`t_test_2`/`t_test_paired`, `anova`, `*_cdf`/`*_pdf`/`*_inv`.
- **Datetime**: `date`, `diff`, `addDays`/`addMonths`/`addYears`, `isWorkday`/`addWorkdays`, `som`/`eom`/`soq`/`eoq`.
- **Matrix**: `matrix`, `determinant`, `inverse`, `solve`, `eigen`, `qr`/`lu`/`svd`/`cholesky`.
- **Units**: `convert(value, from, to)`, `in_units(value, "from->to")`.
- **Constants & closed-form**: `phi`, `pi`, `e` plus PDG/CODATA physical constants; `decompose(value)` to test whether a number is a closed form in phi/pi/e.
